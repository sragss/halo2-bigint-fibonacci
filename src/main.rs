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
}

fn main() {
    let args = Args::parse();

    halo2_bigint_fib::fib_bi::run(args.plot, args.mock, args.num_steps);
}
