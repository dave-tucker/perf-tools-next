use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct Options {}

pub fn pmu(_opts: Options) -> Result<()> {
    Ok(())
}
