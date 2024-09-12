#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod generated;
mod list;
mod pmu_events;
mod stat;
mod version;

use clap::{Parser, Subcommand};

use list::{do_list, ListArgs};
use stat::{do_stat, StatArgs};
use version::{do_version, VersionArgs};

/// Performance analysis tools for Linux
#[derive(Parser)]
#[command(version, about, long_about = None, disable_version_flag = true)]
struct PerfCli {
    #[command(subcommand)]
    command: PerfCommand,
}

#[derive(Subcommand)]
enum PerfCommand {
    /// List all symbolic event types
    ///
    /// This command displays the symbolic event types which can be selected in the various perf commands with the -e option.
    List(ListArgs),
    /// Run a command and gather performance counter statistics
    ///
    /// This command runs a command and gathers performance counter statistics from it.
    Stat(StatArgs),
    /// Display the version of perf binary
    ///
    /// With no options given, the perf version prints the perf version on the standard output.
    /// If the option --build-options is given, then the status of compiled-in libraries are printed on the standard output.
    Version(VersionArgs),
}

fn main() {
    let cli = PerfCli::parse();
    match cli.command {
        PerfCommand::List(args) => do_list(args),
        PerfCommand::Version(args) => do_version(args),
        PerfCommand::Stat(args) => do_stat(args),
    }
    .unwrap();
}
