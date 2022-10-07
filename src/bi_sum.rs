use std::marker::PhantomData;

use halo2_proofs::{
    circuit::SimpleFloorPlanner,
    plonk::{Circuit, ConstraintSystem, Error},
};
use halo2_rsa::big_integer::{BigIntChip, BigIntConfig, BigIntInstructions, UnassignedInteger};
use halo2wrong::halo2::arithmetic::FieldExt;
use maingate::{
    decompose_big, MainGate, MainGateConfig, MainGateInstructions, RangeChip, RangeInstructions,
    RegionCtx,
};
use num_bigint::BigUint;
use rand::{thread_rng, Rng};

struct BigSumCircuit<F: FieldExt> {
    a: BigUint,
    b: BigUint,
    _f: PhantomData<F>,
}

impl<F: FieldExt> BigSumCircuit<F> {
    const LIMB_WIDTH: usize = 64;
    const BITS_LEN: usize = 512;
    fn bigint_chip(&self, config: BigIntConfig) -> BigIntChip<F> {
        BigIntChip::new(config, Self::LIMB_WIDTH, Self::BITS_LEN)
    }
}

impl<F: FieldExt> Circuit<F> for BigSumCircuit<F> {
    type Config = BigIntConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        unimplemented!();
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let main_gate_config = MainGate::<F>::configure(meta);
        let (composition_bit_lens, overflow_bit_lens) = BigIntChip::<F>::compute_range_lens(
            Self::LIMB_WIDTH,
            Self::BITS_LEN / Self::LIMB_WIDTH,
        );
        let range_config = RangeChip::<F>::configure(
            meta,
            &main_gate_config,
            composition_bit_lens,
            overflow_bit_lens,
        );
        BigIntConfig::new(range_config, main_gate_config)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2wrong::halo2::circuit::Layouter<F>,
    ) -> Result<(), Error> {
        let bigint_chip = self.bigint_chip(config);
        let num_limbs = Self::BITS_LEN / Self::LIMB_WIDTH;
        layouter.assign_region(
            || "random add test",
            |region| {
                let offset = 0;
                let ctx = &mut RegionCtx::new(region, offset);
                let sum = &self.a + &self.b;
                //let carry = &all_sum >> Self::BITS_LEN;
                //let base = BigUint::from(1usize) << Self::BITS_LEN;
                //let sum = &all_sum - &carry * &base;

                // Requires 2^9 rows
                let a_limbs = decompose_big::<F>(self.a.clone(), num_limbs, Self::LIMB_WIDTH);
                let a_unassigned = UnassignedInteger::from(a_limbs);
                let b_limbs = decompose_big::<F>(self.b.clone(), num_limbs, Self::LIMB_WIDTH);
                let b_unassigned = UnassignedInteger::from(b_limbs);

                let a_assigned = bigint_chip.assign_integer(ctx, a_unassigned)?;
                let b_assigned = bigint_chip.assign_integer(ctx, b_unassigned)?;
                let added = bigint_chip.add(ctx, &a_assigned, &b_assigned)?;

                let sum_assigned_int = bigint_chip.assign_constant_fresh(ctx, sum)?;

                bigint_chip.assert_equal_fresh(ctx, &sum_assigned_int, &added)?;
                Ok(())
            },
        )?;
        let range_chip = bigint_chip.range_chip();
        range_chip.load_table(&mut layouter)?;
        //range_chip.load_overflow_tables(&mut layouter)?;
        Ok(())
    }
}

#[test]
fn test() {
    use halo2wrong::halo2::dev::MockProver;
    use num_bigint::RandomBits;
    use rand::{thread_rng, Rng};
    fn run<F: FieldExt>() {
        let mut rng = thread_rng();
        let bits_len = BigSumCircuit::<F>::BITS_LEN as u64;
        let mut n = BigUint::default();
        while n.bits() != bits_len {
            n = rng.sample(RandomBits::new(bits_len));
        }
        let a = rng.sample::<BigUint, _>(RandomBits::new(bits_len)) % &n / BigUint::from(2u8);
        let b = rng.sample::<BigUint, _>(RandomBits::new(bits_len)) % &n / BigUint::from(2u8);
        let circuit = BigSumCircuit::<F> {
            a,
            b,
            _f: PhantomData,
        };

        let public_inputs = vec![vec![]];
        let k = 9;
        let prover = match MockProver::run(k, &circuit, public_inputs) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify().is_err(), false);
    }

    use halo2wrong::curves::pasta::Fq as PastaFq;
    run::<PastaFq>();
}
