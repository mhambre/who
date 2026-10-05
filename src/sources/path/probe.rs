use std::io::Write;
use std::process::{Command, Stdio};

use super::context::Compilation;

/// Send source through stdin and isolate all compiler output in a temporary directory.
pub fn compile(compilation: &Compilation, source: &str) -> Result<(bool, String), String> {
    let directory = tempfile::tempdir()
        .map_err(|error| format!("who: cannot create path probe directory: {error}"))?;
    let mut child = Command::new(&compilation.compiler)
        .current_dir(&compilation.directory)
        .args(&compilation.arguments)
        .args([
            "--crate-name",
            "who_path_probe",
            "--crate-type=lib",
            "--emit=metadata",
            "--cap-lints=allow",
            "-o",
        ])
        .arg(directory.path().join("probe.rmeta"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("who: cannot launch path probe compiler: {error}"))?;
    let written = child.stdin.take().unwrap().write_all(source.as_bytes());
    let output = child
        .wait_with_output()
        .map_err(|error| format!("who: cannot wait for path probe compiler: {error}"))?;
    written.map_err(|error| format!("who: cannot send path probe source: {error}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.code().is_none()
        || output
            .status
            .code()
            .is_some_and(|code| code != 0 && code != 1)
    {
        return Err(format!(
            "who: path probe compiler terminated unexpectedly: {}\n{stderr}",
            output.status
        ));
    }
    Ok((output.status.success(), stderr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_imports_use_the_selected_edition() {
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        for edition in ["2015", "2018", "2021", "2024"] {
            let compilation = Compilation {
                compiler: compiler.clone().into(),
                directory: std::env::current_dir().unwrap(),
                arguments: vec!["--edition".into(), edition.into()],
            };
            let (success, stderr) = compile(
                &compilation,
                &super::super::import_source("core::mem::size_of"),
            )
            .unwrap();
            assert!(success, "{edition}: {stderr}");
        }
    }
}
