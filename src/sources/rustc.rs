use std::env;
use std::process::Command;

use semver::Version;

/// Query the compiler selected by Cargo, falling back to rustc on PATH.
pub fn version() -> Result<Version, String> {
    let compiler = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(compiler)
        .arg("--version")
        .output()
        .map_err(|error| format!("who: cannot query rustc: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "who: rustc --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    parse_version(&String::from_utf8_lossy(&output.stdout))
}

/// Preserve prerelease identifiers while discarding rustc's build metadata.
fn parse_version(output: &str) -> Result<Version, String> {
    let mut words = output.split_whitespace();
    if words.next() != Some("rustc") {
        return Err(format!("who: unrecognized compiler version: {output}"));
    }
    Version::parse(words.next().unwrap_or(""))
        .map_err(|error| format!("who: invalid compiler version: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_and_nightly() {
        assert_eq!(
            parse_version("rustc 1.95.0 (abc 2026-01-01)").unwrap(),
            Version::new(1, 95, 0)
        );
        assert_eq!(
            parse_version("rustc 1.96.0-nightly (abc 2026-01-01)")
                .unwrap()
                .pre
                .as_str(),
            "nightly"
        );
        assert!(parse_version("unexpected 1.95.0").is_err());
    }
}
