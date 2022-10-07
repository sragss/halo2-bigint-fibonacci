use std::{marker::PhantomData, time::Instant};

use halo2_proofs::{
    circuit::{SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error, ProvingKey, verify_proof}, transcript::{Blake2bWrite, Challenge255, TranscriptWriterBuffer, Blake2bRead, TranscriptReadBuffer}, poly::{ipa::{commitment::{IPACommitmentScheme, ParamsIPA}, multiopen::ProverIPA, strategy::SingleStrategy}, commitment::ParamsProver, VerificationStrategy},
};
use halo2_rsa::big_integer::{BigIntChip, BigIntConfig, BigIntInstructions, UnassignedInteger};
use halo2curves::pasta::EqAffine;
use halo2wrong::halo2::arithmetic::FieldExt;
use maingate::{
    decompose_big, MainGate, RangeChip, RangeInstructions,
    RegionCtx,
};
use num_bigint::BigUint;
use rand_core::OsRng;

use crate::utils;

struct BigFibCircuit<F: FieldExt> {
    init_a: BigUint,
    init_b: BigUint,
    final_sum: BigUint,
    n: usize,
    _f: PhantomData<F>,
}

impl<F: FieldExt> BigFibCircuit<F> {
    const LIMB_WIDTH: usize = 64;
    const BITS_LEN: usize = 512;
    fn bigint_chip(&self, config: BigIntConfig) -> BigIntChip<F> {
        BigIntChip::new(config, Self::LIMB_WIDTH, Self::BITS_LEN)
    }
}

impl<F: FieldExt> Circuit<F> for BigFibCircuit<F> {
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
            || "fib_step",
            |region| {
                let offset = 0;
                let ctx = &mut RegionCtx::new(region, offset);

                // let mut sum = &self.init_a + &self.init_b;

                let a_limbs = decompose_big::<F>(self.init_a.clone(), num_limbs, Self::LIMB_WIDTH);
                let b_limbs = decompose_big::<F>(self.init_b.clone(), num_limbs, Self::LIMB_WIDTH);
                let a_unassigned = UnassignedInteger::from(a_limbs);
                let b_unassigned = UnassignedInteger::from(b_limbs);

                let mut a_assigned = bigint_chip.assign_integer(ctx, a_unassigned)?;
                let mut b_assigned = bigint_chip.assign_integer(ctx, b_unassigned)?;
                let mut c_assigned = bigint_chip.add(ctx, &a_assigned, &b_assigned)?;

                // Single add of 512 bit numbers requires 2^9 rows
                for _ in 2..self.n {
                    // Move pointers: 
                    // A[n] = B[n-1]
                    // B[n] = C[n-1]
                    a_assigned = b_assigned;
                    b_assigned = c_assigned;

                    // C[n] = A[n] + B[n]
                    c_assigned = bigint_chip.add(ctx, &a_assigned, &b_assigned)?;

                    // TODO: Do I need intermedite sum equality checks?
                    // let sum_assigned_int = bigint_chip.assign_constant_fresh(ctx, sum)?;
                    // bigint_chip.assert_equal_fresh(ctx, &sum_assigned_int, &c_assigned)?;
                }

                // TODO: This doesn't work yet! I think we need to do something with public inputs
                // let sum_assigned_int = bigint_chip.assign_constant_fresh(ctx, self.final_sum.clone())?;
                // bigint_chip.assert_equal_fresh(ctx, &sum_assigned_int, &c_assigned)?;
                Ok(())
            },
        )?;
        let range_chip = bigint_chip.range_chip();
        range_chip.load_table(&mut layouter)?;
        //range_chip.load_overflow_tables(&mut layouter)?;
        Ok(())
    }
}


use halo2wrong::halo2::dev::MockProver;
use halo2wrong::curves::pasta::Fp as PastaFp;
use halo2_proofs::plonk::{create_proof, keygen_vk, keygen_pk};

pub fn run(plot: bool, mock: bool, n: usize) {
    let final_sum = utils::big_fib_calc(n);

    let circuit = BigFibCircuit::<PastaFp> {
        init_a: BigUint::from(0u8),
        init_b: BigUint::from(1u8),
        n,
        final_sum,
        _f: PhantomData,
    };

    let public_inputs = vec![vec![]];
    let k = calc_k(n);

    if mock {
        let prover = match MockProver::run(k, &circuit, public_inputs) {
            Ok(prover) => prover,
            Err(e) => panic!("{:#?}", e),
        };
        assert_eq!(prover.verify().is_err(), false);
    } else {
        let rng = OsRng;
        let mut transcript = Blake2bWrite::<_, _, Challenge255<EqAffine>>::init(vec![]);

        let pre_keygen = Instant::now();
        let (params, pk) = keygen(k);
        let keygen_time = pre_keygen.elapsed();
        let pre_proof = Instant::now();
        match create_proof::<IPACommitmentScheme<EqAffine>, ProverIPA<EqAffine>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[&[]]],
            rng,
            &mut transcript,
        ) {
            Ok(()) => (),
            Err(e) => panic!("{:#?}", e),
        }
        let proof = transcript.finalize();
        let proof_time = pre_proof.elapsed();

        let strategy = SingleStrategy::new(&params);
        let mut ver_transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        match verify_proof(&params, pk.get_vk(), strategy, &[&[&[]]], &mut ver_transcript) {
            Ok(()) => println!("Verified!"),
            Err(e) => panic!("{:#?}", e)
        }

        println!("Keygen time: {}ms", keygen_time.as_millis());
        println!("Proof time: {}ms", proof_time.as_millis());
    }

    if plot {
        use plotters::prelude::*;
        let plot_name = "plots/Fib_bi.png";
        let root = BitMapBackend::new(plot_name, (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(plot_name, ("sans-serif", 60)).unwrap();

        // halo2_proofs::dev::CircuitLayout::default()
        //     .show_labels(true)
        //     .show_equality_constraints(true)
        //     .render(k, &circuit, &root)
        //     .unwrap();
        println!("Plot rendered to {}", plot_name);
    }
}

fn keygen(k: u32) -> (ParamsIPA<EqAffine>, ProvingKey<EqAffine>) {
    let params: ParamsIPA<EqAffine> = ParamsIPA::new(k);
    let empty_circuit: BigFibCircuit<PastaFp> = BigFibCircuit {
        init_a: BigUint::from(0u8),
        init_b: BigUint::from(0u8),
        final_sum: BigUint::from(0u8), // TODO: idk
        n: 0,
        _f: PhantomData::default(),
    };
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");
    (params, pk)
}

// Rough estimate
fn calc_k(fib_steps: usize) -> u32 {
    let n = 500 * fib_steps;
    return fast_math::log2(n as f32).ceil() as u32 + 1;
}

#[test]
fn test() {
    run(false, true, 50);
}
