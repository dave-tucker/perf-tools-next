mod codegen;
mod pmu;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct XtaskOptions {
    #[clap(subcommand)]
    command: Subcommand,
}

#[derive(Parser)]
enum Subcommand {
    Codegen(codegen::Options),
    Pmu(pmu::Options),
}

fn main() -> Result<()> {
    let XtaskOptions { command } = Parser::parse();

    match command {
        Subcommand::Codegen(opts) => codegen::codegen(opts),
        Subcommand::Pmu(opts) => pmu::pmu(opts),
    }
}
