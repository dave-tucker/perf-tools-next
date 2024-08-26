use std::{env::var, fs::read_dir, io::Write, os::raw::c_uint, path::PathBuf};

use clap::{ArgAction, Args, ValueEnum};
use pager::Pager;
use textwrap::wrap;

use crate::generated::uapi::{
    perf_hw_id_PERF_COUNT_HW_BRANCH_INSTRUCTIONS, perf_hw_id_PERF_COUNT_HW_BRANCH_MISSES,
    perf_hw_id_PERF_COUNT_HW_BUS_CYCLES, perf_hw_id_PERF_COUNT_HW_CACHE_MISSES,
    perf_hw_id_PERF_COUNT_HW_CACHE_REFERENCES, perf_hw_id_PERF_COUNT_HW_CPU_CYCLES,
    perf_hw_id_PERF_COUNT_HW_INSTRUCTIONS, perf_hw_id_PERF_COUNT_HW_REF_CPU_CYCLES,
    perf_hw_id_PERF_COUNT_HW_STALLED_CYCLES_BACKEND,
    perf_hw_id_PERF_COUNT_HW_STALLED_CYCLES_FRONTEND, perf_sw_ids_PERF_COUNT_SW_ALIGNMENT_FAULTS,
    perf_sw_ids_PERF_COUNT_SW_BPF_OUTPUT, perf_sw_ids_PERF_COUNT_SW_CGROUP_SWITCHES,
    perf_sw_ids_PERF_COUNT_SW_CONTEXT_SWITCHES, perf_sw_ids_PERF_COUNT_SW_CPU_CLOCK,
    perf_sw_ids_PERF_COUNT_SW_CPU_MIGRATIONS, perf_sw_ids_PERF_COUNT_SW_DUMMY,
    perf_sw_ids_PERF_COUNT_SW_EMULATION_FAULTS, perf_sw_ids_PERF_COUNT_SW_PAGE_FAULTS,
    perf_sw_ids_PERF_COUNT_SW_PAGE_FAULTS_MAJ, perf_sw_ids_PERF_COUNT_SW_PAGE_FAULTS_MIN,
    perf_sw_ids_PERF_COUNT_SW_TASK_CLOCK,
};

struct EventInfo {
    _id: c_uint,
    symbol: &'static str,
    alias: Option<&'static str>,
}

static HW_EVENTS: &[EventInfo] = &[
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_CPU_CYCLES,
        symbol: "cpu-cycles",
        alias: Some("cycles"),
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_INSTRUCTIONS,
        symbol: "instructions",
        alias: None,
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_CACHE_REFERENCES,
        symbol: "cache-references",
        alias: None,
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_CACHE_MISSES,
        symbol: "cache-misses",
        alias: None,
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_BRANCH_INSTRUCTIONS,
        symbol: "branch-instructions",
        alias: Some("branches"),
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_BRANCH_MISSES,
        symbol: "branch-misses",
        alias: None,
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_BUS_CYCLES,
        symbol: "bus-cycles",
        alias: None,
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_STALLED_CYCLES_FRONTEND,
        symbol: "stalled-cycles-frontend",
        alias: Some("idle-cycles-frontend"),
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_STALLED_CYCLES_BACKEND,
        symbol: "stalled-cycles-backend",
        alias: Some("idle-cycles-backend"),
    },
    EventInfo {
        _id: perf_hw_id_PERF_COUNT_HW_REF_CPU_CYCLES,
        symbol: "ref-cycles",
        alias: None,
    },
];

static SW_EVENTS: &[EventInfo] = &[
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_CPU_CLOCK,
        symbol: "cpu-clock",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_TASK_CLOCK,
        symbol: "task-clock",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_PAGE_FAULTS,
        symbol: "page-faults",
        alias: Some("faults"),
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_CONTEXT_SWITCHES,
        symbol: "context-switches",
        alias: Some("cs"),
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_CPU_MIGRATIONS,
        symbol: "cpu-migrations",
        alias: Some("migrations"),
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_PAGE_FAULTS_MIN,
        symbol: "minor-faults",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_PAGE_FAULTS_MAJ,
        symbol: "major-faults",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_ALIGNMENT_FAULTS,
        symbol: "alignment-faults",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_EMULATION_FAULTS,
        symbol: "emulation-faults",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_DUMMY,
        symbol: "dummy",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_BPF_OUTPUT,
        symbol: "bpf-output",
        alias: None,
    },
    EventInfo {
        _id: perf_sw_ids_PERF_COUNT_SW_CGROUP_SWITCHES,
        symbol: "cgroup-switches",
        alias: None,
    },
];

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub(crate) struct ListArgs {
    /// Print extra event descriptions. (default)
    #[arg(short = 'd', long = "desc")]
    pub(crate) _desc: bool,

    /// Don’t print descriptions.
    #[arg(long = "no-desc", conflicts_with = "_desc", default_value = "true", action = ArgAction::SetFalse)]
    pub(crate) desc: bool,

    /// Print longer event descriptions.
    #[arg(short = 'v', long, conflicts_with = "_desc", default_value = "false")]
    pub(crate) long_desc: bool,

    /// Enable debugging output.
    #[arg(long)]
    pub(crate) debug: bool,

    /// Print how named events are resolved internally into perf events, and also any extra expressions computed by perf stat.
    #[arg(long)]
    pub(crate) details: bool,

    /// Print deprecated events. By default the deprecated events are hidden.
    #[arg(long)]
    pub(crate) deprecated: bool,

    /// Print PMU events and metrics limited to the specific PMU name. (e.g. --unit cpu, --unit msr, --unit cpu_core, --unit cpu_atom)
    #[arg(long)]
    pub(crate) unit: Option<String>,

    /// Output in JSON format.
    #[arg(short, long)]
    pub(crate) json: bool,

    /// Output file name. By default output is written to stdout.
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,

    #[arg(value_enum)]
    event: Option<EventType>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EventType {
    HW,
    SW,
    Cache,
    Tracepoint,
    PMU,
    SDT,
    Metric,
    #[clap(name = "metricgroup")]
    MetricGroup,
    #[clap(name = "event_glob")]
    EventGlob,
    #[cfg(feature = "libpfm4")]
    Pfm,
}

pub(crate) fn do_list(args: ListArgs) -> anyhow::Result<()> {
    Pager::with_default_pager("less").setup();
    println!("List of pre-defined events (to be used in -e or -M):\n");

    match args.event {
        Some(EventType::HW) => {
            print_symbol_events(HW_EVENTS, "Hardware Events")?;
        }
        Some(EventType::SW) => {
            print_symbol_events(SW_EVENTS, "Hardware Events")?;
        }
        Some(EventType::Cache) => {
            print_cache_events()?;
        }
        Some(EventType::Tracepoint) => {
            print_tracepoint_events(&args)?;
        }
        Some(EventType::PMU) => {
            print_pmu_events()?;
        }
        Some(EventType::SDT) => {
            print_sdt_events()?;
        }
        Some(EventType::Metric) => {
            print_metric_events()?;
        }
        Some(EventType::MetricGroup) => {
            print_metric_group_events()?;
        }
        Some(EventType::EventGlob) => {
            print_event_glob_events()?;
        }
        #[cfg(feature = "libpfm4")]
        Some(EventType::Pfm) => {
            print_pfm_events()?;
        }
        None => {
            println!("Hardware events:");
            print_symbol_events(HW_EVENTS, "Hardware Events")?;
            println!("\nSoftware events:");
            print_symbol_events(SW_EVENTS, "Software Events")?;
            println!("\nCache events:");
            print_cache_events()?;
            println!("\nTracepoint events:");
            print_tracepoint_events(&args)?;
            println!("\nPMU events:");
            print_pmu_events()?;
            println!("\nSDT events:");
            print_sdt_events()?;
            println!("\nMetric events:");
            print_metric_events()?;
            println!("\nMetric group events:");
            print_metric_group_events()?;
            println!("\nEvent glob events:");
            print_event_glob_events()?;
            #[cfg(feature = "libpfm4")]
            {
                println!("\nPfm events:");
                print_pfm_events()?;
            }
        }
    }
    print_tracepoint_events(&args)?;
    Ok(())
}

fn print_symbol_events(events: &[EventInfo], event_type: &'static str) -> anyhow::Result<()> {
    let mut result = vec![];
    for event in events {
        if let Some(out) = print_event(
            &ListArgs {
                _desc: false,
                desc: true,
                long_desc: false,
                debug: false,
                details: false,
                deprecated: false,
                unit: None,
                json: false,
                output: None,
                event: None,
            },
            None,
            Some(event.symbol.to_string()),
            event.alias.map(|s| s.to_string()),
            false,
            Some(event_type),
            None,
            None,
            None,
        ) {
            result.push(out);
        }
    }

    result.sort();
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    for r in result {
        if let Err(e) = writeln!(lock, "{}", r) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                break;
            }
        }
    }
    Ok(())
}

fn print_cache_events() -> anyhow::Result<()> {
    Ok(())
}

fn print_pmu_events() -> anyhow::Result<()> {
    Ok(())
}

fn print_sdt_events() -> anyhow::Result<()> {
    Ok(())
}

fn print_metric_events() -> anyhow::Result<()> {
    Ok(())
}

fn print_metric_group_events() -> anyhow::Result<()> {
    Ok(())
}

fn print_event_glob_events() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(feature = "libpfm4")]
fn print_pfm_events() -> anyhow::Result<()> {
    Ok(())
}

fn get_columns() -> usize {
    var("COLUMNS")
        .unwrap_or("80".to_string())
        .parse()
        .unwrap_or(80)
}

fn print_event(
    args: &ListArgs,
    pmu_name: Option<String>,
    event_name: Option<String>,
    event_alias: Option<String>,
    deprecated: bool,
    event_type_desc: Option<&'static str>,
    desc: Option<String>,
    long_desc: Option<String>,
    encoding_desc: Option<String>,
) -> Option<String> {
    if !args.deprecated && deprecated {
        return None;
    }
    let mut buf = String::new();
    buf.push_str("  ");

    if let Some(event_name) = event_name {
        buf.push_str(&event_name);
    }
    if let Some(event_alias) = event_alias {
        buf.push_str(&format!(" OR {}", event_alias));
    }
    if let Some(event_type_desc) = event_type_desc {
        if buf.len() < 53 {
            buf.push_str(&" ".repeat(53 - buf.len()));
        }
        buf.push_str(&format!("[{}]", event_type_desc));
    }
    if args.long_desc {
        if let Some(long_desc) = long_desc {
            let lines =
                wrap(&long_desc, get_columns() - 8)
                    .into_iter()
                    .fold(String::new(), |acc, line| {
                        if acc.is_empty() {
                            acc + &format!("{}", line)
                        } else {
                            acc + &format!("\n{:>8} {}", "", line)
                        }
                    });
            buf.push_str(&format!("\n{:>8}{}]", "[", lines));
        }
    } else if args.desc {
        if let Some(desc) = desc {
            let desc = if let Some(pmu_name) = pmu_name {
                if pmu_name == "default_core".to_string() {
                    if desc.chars().last().unwrap() != '.' {
                        format!("{}. Unit: {}", desc, pmu_name)
                    } else {
                        format!("{} Unit: {}", desc, pmu_name)
                    }
                } else {
                    desc
                }
            } else {
                desc
            };
            let desc =
                wrap(&desc, get_columns() - 8)
                    .into_iter()
                    .fold(String::new(), |acc, line| {
                        if acc.is_empty() {
                            acc + &format!("{}", line)
                        } else {
                            acc + &format!("\n{:>8} {}", "", line)
                        }
                    });

            buf.push_str(&format!("\n{:>8}{}]\n", "[", desc));
        }
    }
    if args.details {
        if let Some(encoding_desc) = encoding_desc {
            buf.push_str(&format!("\n{:>8} {}", "", encoding_desc));
        }
    }

    Some(buf)
}

fn print_tracepoint_events(args: &ListArgs) -> anyhow::Result<()> {
    let tracepoint_dir = PathBuf::from("/sys/kernel/debug/tracing/events");

    let mut tracepoints = vec![];

    for entry in read_dir(tracepoint_dir)? {
        let entry = entry?;
        let event_category = entry.path();
        let category_str = event_category
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_owned();
        if event_category.is_dir() {
            for entry in read_dir(event_category.clone())? {
                let entry = entry?;
                let event_name = entry.path();
                if event_name.is_dir() {
                    let name_str = event_name.file_name().unwrap().to_string_lossy();
                    let event_file = event_name.join("id");
                    let event_desc = if event_file.exists() {
                        let id = std::fs::read_to_string(event_file).unwrap();
                        let id: u64 = id.trim().parse().unwrap();
                        Some(format!("tracepoint/config={:#x}/", id))
                    } else {
                        None
                    };

                    if let Some(out) = print_event(
                        args,
                        None,
                        Some(format!("{}:{}", category_str, name_str)),
                        None,
                        false,
                        Some("Tracepoint event"),
                        None,
                        None,
                        event_desc,
                    ) {
                        tracepoints.push(out);
                    }
                }
            }
        }
    }

    tracepoints.sort();
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    for tracepoint in tracepoints {
        if let Err(e) = writeln!(lock, "{}", tracepoint) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_event_hw() {
        let s = print_event(
            &crate::list::ListArgs {
                _desc: false,
                desc: true,
                long_desc: false,
                debug: false,
                details: false,
                deprecated: false,
                unit: None,
                json: false,
                output: None,
                event: None,
            },
            None,
            Some("cpu-cycles".to_string()),
            Some("cycles".to_string()),
            false,
            Some("Hardware event"),
            None,
            None,
            None,
        );
        assert_eq!(
            s,
            Some(
                "cpu-cycles OR cycles                                 [Hardware event]".to_string()
            )
        );
    }

    #[test]
    fn test_print_event_tracepoint() {
        let s = print_event(
            &crate::list::ListArgs {
                _desc: false,
                desc: true,
                long_desc: false,
                debug: false,
                details: true,
                deprecated: false,
                unit: None,
                json: false,
                output: None,
                event: None,
            },
            None,
            Some("sched:sched_switch".to_string()),
            None,
            false,
            Some("Tracepoint event"),
            None,
            None,
            Some("tracepoint/config=0x1/".to_string()),
        );
        assert_eq!(
            s,
            Some(
                "sched:sched_switch                                   [Tracepoint event]\n         tracepoint/config=0x1/"
                    .to_string()),
        );
    }
}
