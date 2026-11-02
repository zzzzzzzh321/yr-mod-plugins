//! Offline Component Model packaging. This tool neither executes the module nor
//! links WASI adapters; runtime identity/import admission remains the host's job.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use wit_component::ComponentEncoder;

const MAX_WASM_BYTES: usize = 16 * 1024 * 1024;
const CORE_MODULE_HEADER: &[u8; 8] = b"\0asm\x01\0\0\0";
pub const DEFAULT_PROGRAM: &str = "gameplay.wasm";

fn read_core_module(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let metadata = file.metadata().context("read input metadata")?;
    ensure!(metadata.is_file(), "input must be a regular core Wasm file");
    ensure!(
        metadata.len() <= MAX_WASM_BYTES as u64,
        "input exceeds 16 MiB"
    );
    // A concurrent writer cannot make this read unbounded between metadata
    // inspection and admission; one extra byte detects growth past the limit.
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_WASM_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .context("read core module")?;
    ensure!(
        bytes.len() <= MAX_WASM_BYTES,
        "input grew beyond 16 MiB while reading"
    );
    ensure!(
        bytes.starts_with(CORE_MODULE_HEADER),
        "input must be a core Wasm module; build for wasm32-unknown-unknown"
    );
    Ok(bytes)
}

/// Encode a bounded core module into a new publication directory. Returns the
/// lowercase SHA-256 of `gameplay.wasm`; existing output is never overwritten.
pub fn package(input: &Path, output: &Path) -> Result<String> {
    package_with_program(input, output, DEFAULT_PROGRAM)
}

/// Package with a portable basename used by both the file and declaration.
/// Paths and Windows device names are refused before creating any output.
pub fn package_with_program(input: &Path, output: &Path, program: &str) -> Result<String> {
    let stem = program.split('.').next().unwrap_or_default();
    let device = stem.to_ascii_uppercase();
    ensure!(
        program.len() <= 128
            && program.ends_with(".wasm")
            && program
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && program
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            && !matches!(device.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            && !(device.len() == 4
                && (device.starts_with("COM") || device.starts_with("LPT"))
                && matches!(device.as_bytes()[3], b'1'..=b'9')),
        "program must be a portable .wasm basename (at most 128 ASCII characters)"
    );
    ensure!(
        output.file_name().is_some(),
        "output must name a new directory"
    );
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    ensure!(parent.is_dir(), "output parent must already exist");
    match fs::symlink_metadata(output) {
        Ok(_) => anyhow::bail!("output already exists; choose a new directory"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("inspect output directory"),
    }
    let input = read_core_module(input)?;
    let component = ComponentEncoder::default()
        .module(&input)
        .context("read component metadata from core module")?
        .validate(true)
        .encode()
        .context("encode and validate component without WASI adapters")?;
    ensure!(
        component.len() <= MAX_WASM_BYTES,
        "encoded component exceeds 16 MiB"
    );
    let sha256 = format!("{:x}", Sha256::digest(&component));
    let declaration = serde_json::to_vec_pretty(&serde_json::json!({
        "api_version": vera_mod_api::API_VERSION,
        "program": program,
        "sha256": sha256,
    }))?;
    // Failed reads/encoding/validation leave no output directory. create_dir
    // also closes the preflight race without ever accepting existing output.
    fs::create_dir(output).context("create new package directory")?;
    write_new(&output.join(program), &component)?;
    write_new(&output.join("vera-mod.json"), &declaration)?;
    Ok(sha256)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("vera-mod-pack-test-{}-{stamp}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            // Only exact files beneath this test's newly created directory.
            for name in [
                "input.wasm",
                "out/marker",
                "out/gameplay.wasm",
                "out/vera20k-ares.wasm",
                "out/vera-mod.json",
            ] {
                let _ = fs::remove_file(self.0.join(name));
            }
            let _ = fs::remove_dir(self.0.join("out"));
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn invalid_core_leaves_no_output() {
        let dir = TestDir::new();
        let input = dir.0.join("input.wasm");
        fs::write(&input, b"not a core module").unwrap();
        assert!(package(&input, &dir.0.join("out")).is_err());
        assert!(!dir.0.join("out").exists());
    }

    #[test]
    fn existing_output_is_preserved() {
        let dir = TestDir::new();
        let out = dir.0.join("out");
        fs::create_dir(&out).unwrap();
        fs::write(out.join("marker"), b"retain").unwrap();
        assert!(package(&dir.0.join("input.wasm"), &out).is_err());
        assert_eq!(fs::read(out.join("marker")).unwrap(), b"retain");
        assert!(!out.join("gameplay.wasm").exists());
    }

    #[test]
    fn oversized_input_is_rejected_before_output_creation() {
        let dir = TestDir::new();
        let input = dir.0.join("input.wasm");
        let file = File::create(&input).unwrap();
        file.set_len(MAX_WASM_BYTES as u64 + 1).unwrap();
        drop(file);
        assert!(package(&input, &dir.0.join("out")).is_err());
        assert!(!dir.0.join("out").exists());
    }

    #[test]
    fn named_component_matches_its_declaration_and_digest() {
        let dir = TestDir::new();
        let input = dir.0.join("input.wasm");
        fs::write(&input, CORE_MODULE_HEADER).unwrap();
        let out = dir.0.join("out");
        let digest = package_with_program(&input, &out, "vera20k-ares.wasm").unwrap();
        let declaration: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("vera-mod.json")).unwrap()).unwrap();
        let component = fs::read(out.join("vera20k-ares.wasm")).unwrap();
        assert_eq!(declaration["program"], "vera20k-ares.wasm");
        assert_eq!(declaration["sha256"], digest);
        assert_eq!(digest, format!("{:x}", Sha256::digest(component)));
        assert!(!out.join(DEFAULT_PROGRAM).exists());
    }

    #[test]
    fn invalid_program_paths_are_rejected_before_output_creation() {
        let dir = TestDir::new();
        let input = dir.0.join("input.wasm");
        fs::write(&input, CORE_MODULE_HEADER).unwrap();
        let out = dir.0.join("out");
        for name in ["../outside.wasm", "sub\\outside.wasm", "CON.wasm"] {
            let error = package_with_program(&input, &out, name).unwrap_err();
            assert!(error.to_string().contains("portable .wasm basename"));
            assert!(!out.exists());
        }
    }
}
