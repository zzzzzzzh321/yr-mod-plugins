//! Command-line adapter for the offline SDK component packager.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: vera-mod-pack INPUT_CORE_WASM OUTPUT_NEW_DIR [--program NAME.wasm]")?,
    );
    let output = PathBuf::from(args.next().context("missing OUTPUT_NEW_DIR")?);
    let program = match args.next() {
        Some(flag) => {
            ensure!(flag == "--program", "expected --program NAME.wasm");
            args.next()
                .context("missing NAME.wasm")?
                .into_string()
                .map_err(|_| anyhow::anyhow!("program name must be Unicode"))?
        }
        None => vera_mod_pack::DEFAULT_PROGRAM.to_owned(),
    };
    ensure!(
        args.next().is_none(),
        "usage: vera-mod-pack INPUT_CORE_WASM OUTPUT_NEW_DIR [--program NAME.wasm]"
    );
    let sha256 = vera_mod_pack::package_with_program(&input, &output, &program)?;
    println!(
        "{}",
        serde_json::json!({"directory": output, "program": program, "sha256": sha256})
    );
    Ok(())
}
