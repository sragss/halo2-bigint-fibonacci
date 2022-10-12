# Halo2 Fibonacci Calculation
Bigint version of Fibonacci Sequence calculation in [Halo2](https://zcash.github.io/halo2/). Non-bigint version [here](https://github.com/sragss/halo2-fibonacci). 

It's a bit of a mess at the moment... wanted some early results.

Uses the [PSE halo2 fork](https://github.com/privacy-scaling-explorations/halo2) which allows IPA or KZG backends.

All Halo2 BigInt math thanks to Sora Suegami's [halo2_rsa](https://github.com/SoraSuegami/halo2_rsa).

# Current Results
| n | k | vkey | proof | verification |
|---|---|---|---|---|
| 100 | 17 | 42s | 9.3s | 173ms |
| 150 | 17 | 42s | 9.79s | 166ms |
| 200 | 18 | 90.3s | 18.4s | 307ms |
| 250 | 18 | 89.2s | 19.89s | 317ms |
| 350 | 19 | 196.5s | 35.9s | 605ms |
| 450 | 20 | 373.5s | 69.9s | 1080ms |
| 550 | 21 | 836.6s | 134.6s | 1980ms |

(2021 M1 MBP)

*Note `k` has very roughly been experimentally derived.*

# Todo
- [ ] `FibCircuit.final_step` should be a public input + The constraint system creates some instance columns that need to be satisfied by proof inputs to make the actual equality constraint work.
- [ ] Investigate K column (may be too big)
- [ ] Fix `--plot` flag

# Cmds
`cargo run --release`
```
Usage: halo2_bigint_fib [OPTIONS]

Options:
      --mock                   Run mock prover or actual (with keygen and things)
      --plot                   Create plot of circuit layout
      --num-steps <NUM_STEPS>  Number of fibonacci steps [default: 180]
  -h, --help                   Print help information
  -V, --version                Print version information
```

Tests: `cargo test`