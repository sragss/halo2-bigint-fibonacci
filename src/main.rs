use clap::{arg, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run mock prover or actual (with keygen and things)
    #[arg(long)]
    mock: bool,

    /// Create plot of circuit layout
    #[arg(long, default_value_t = false)]
    plot: bool,

    /// Number of fibonacci steps
    #[arg(long, default_value_t = 180)]
    num_steps: usize,

    /// Increment n and k, trial and error find optimal k
    #[arg(long, default_value_t = false)]
    k_test: bool,
}

fn main() {
    let args = Args::parse();


    if args.k_test {
        experimental_k();

    } else {
        halo2_bigint_fib::fib_bi::run(args.plot, args.mock, args.num_steps, 100).unwrap();
    }
}

fn experimental_k() {
        // Run through loop, increasing n, k
        let mut n = 50;
        let max_n = 5000;
        let mut k = 5;

        while n < max_n {
            println!("Running for n = {} with k = {}", n, k);
            let success = match halo2_bigint_fib::fib_bi::run(false, true, n, k) {
                Ok(_) => true,
                Err(_) => false,
            };
            if success {
                println!("[Success] for n = {} with k = {}", n, k);

                n += 50;
            } else {
                k += 1;
                println!("[Fail] for n = {} with k = {}", n, k);
            }
        }
}