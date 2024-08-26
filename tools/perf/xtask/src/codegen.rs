use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::Write;
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
pub struct Options {}

pub fn codegen(_opts: Options) -> Result<()> {
    let in_dir = PathBuf::from("perf-rs/include");
    let out_dir = PathBuf::from("perf-rs/src/generated");

    let types = [
        "perf_sw_ids",
        "perf_hw_id",
        "perf_hw_cache_id",
        "perf_hw_cache_op_id",
        "perf_hw_cache_op_result_id",
        "perf_type_id",
    ];
    let vars = ["PERF_TYPE_.*,", "PERF_COUNT_.*,"];

    let mut bindgen = bindgen::builder()
        .clang_arg(format!(
            "-I{}",
            PathBuf::from("../../usr/include")
                .canonicalize()
                .unwrap()
                .to_string_lossy()
        ))
        .header(in_dir.join("bindings.h").to_string_lossy());

    for x in &types {
        bindgen = bindgen.allowlist_type(x);
    }

    for x in &vars {
        bindgen = bindgen.allowlist_var(x)
    }

    let bindings = bindgen
        .generate()
        .map_err(|op| anyhow!("bindgen failed - {op}"))?
        .to_string();

    let mut file = File::create(out_dir.join("uapi.rs"))?;
    file.write_all(&bindings.as_bytes())?;

    Ok(())
}
