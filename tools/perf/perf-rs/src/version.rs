use clap::Args;
use colored::Colorize;
use shadow_rs::shadow;

shadow!(build);

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub(crate) struct VersionArgs {
    /// Prints the status of compiled-in libraries on the standard output.
    #[arg(long, action, default_value = "false")]
    pub(crate) build_options: bool,
}

macro_rules! print_supported {
    ($feature:literal,$var:literal) => {
        if cfg!(feature = $feature) {
            println!("{:>22}: [{:^5}] # {}", $feature, "on".green(), $var);
        } else {
            println!("{:>22}: [{:^5}] # {}", $feature, "OFF".red(), $var);
        }
    };
}

pub(crate) fn do_version(args: VersionArgs) -> anyhow::Result<()> {
    let version = format!(
        "perf version {}.g{}",
        build::PKG_VERSION,
        build::SHORT_COMMIT
    );
    println!("{}", version);
    if args.build_options {
        print_supported!("dwarf", "HAVE_DWARF_SUPPORT");
        print_supported!("dwarf_getlocations", "HAVE_DWARF_GETLOCATIONS_SUPPORT");
        print_supported!("syscall_table", "HAVE_SYSCALL_TABLE_SUPPORT");
        print_supported!("libbfd", "HAVE_LIBBFD_SUPPORT");
        print_supported!("debuginfod", "HAVE_DEBUGINFOD_SUPPORT");
        print_supported!("libelf", "HAVE_LIBELF_SUPPORT");
        print_supported!("libnuma", "HAVE_LIBNUMA_SUPPORT");
        print_supported!("numa_num_possible_cpus", "HAVE_NUMA_NUM_POSSIBLE_NODES");
        print_supported!("libperl", "HAVE_LIBPERL_SUPPORT");
        print_supported!("libpython", "HAVE_LIBPYTHON_SUPPORT");
        print_supported!("libslang", "HAVE_SLANG_SUPPORT");
        print_supported!("libcrypto", "HAVE_LIBCRYPTO_SUPPORT");
        print_supported!("libunwind", "HAVE_LIBUNWIND_SUPPORT");
        print_supported!("libdw-dwarf-unwind", "HAVE_DWARF_SUPPORT");
        print_supported!("libcapstone", "HAVE_LIBCAPSTONE_SUPPORT");
        print_supported!("zlib", "HAVE_ZLIB_SUPPORT");
        print_supported!("lzma", "HAVE_LZMA_SUPPORT");
        print_supported!("get_cpuid", "HAVE_AUXTRACE_SUPPORT");
        print_supported!("bpf", "HAVE_LIBBPF_SUPPORT");
        print_supported!("aio", "HAVE_AIO_SUPPORT");
        print_supported!("zstd", "HAVE_ZSTD_SUPPORT");
        print_supported!("libpfm4", "HAVE_LIBPFM");
        print_supported!("libtraceevent", "HAVE_LIBTRACEEVENT");
        print_supported!("bpf_skeletons", "HAVE_BPF_SKEL");
        print_supported!("dwarf-unwind-support", "HAVE_DWARF_UNWIND_SUPPORT");
        print_supported!("libopencsd", "HAVE_CSTRACE_SUPPORT");
    }
    Ok(())
}
