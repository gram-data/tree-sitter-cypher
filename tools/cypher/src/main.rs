use clap::{Parser, Subcommand};

mod dispatch;
mod lint;
mod rules;
mod types;

#[derive(Parser)]
#[command(name = "cypher", version = env!("CARGO_PKG_VERSION"), about = "Unified cypher CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint .cypher files for parse and semantic errors
    Lint(lint::LintArgs),
    /// Dispatch to an external cypher-<name> binary on PATH
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Lint(args) => lint::run(args),
        Commands::External(args) => dispatch::run(&args),
    };
    std::process::exit(code);
}
