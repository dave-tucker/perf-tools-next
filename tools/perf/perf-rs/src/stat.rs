use clap::Args;

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub(crate) struct StatArgs {
    event: Option<String>,
}

pub(crate) fn do_stat(_: StatArgs) -> anyhow::Result<()> {
    Ok(())
}
