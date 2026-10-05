use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Compilation {
    pub compiler: PathBuf,
    pub directory: PathBuf,
    pub arguments: Vec<OsString>,
}

impl Compilation {
    /// Capture the actual compiler executable and its resolution context.
    pub fn current() -> Result<Self, String> {
        let arguments = expand_response_files(env::args_os().skip(1).collect(), 0)?;
        if !arguments.iter().any(|argument| {
            argument == "--crate-name"
                || argument
                    .to_str()
                    .is_some_and(|value| value.starts_with("--crate-name="))
        }) {
            return Err("who: path() cannot capture the active rustc invocation. Compile through Cargo with rustc".into());
        }
        let executable = env::current_exe()
            .map_err(|error| format!("who: cannot locate the active compiler: {error}"))?;
        let (compiler, driver_cfg) = compiler_for_driver(executable);
        let mut arguments = resolution_arguments(&arguments)?;
        if let Some(cfg) = driver_cfg {
            arguments.extend([OsString::from("--cfg"), OsString::from(cfg)]);
        }
        Ok(Self {
            compiler,
            directory: env::current_dir()
                .map_err(|error| format!("who: cannot locate the compiler directory: {error}"))?,
            arguments,
        })
    }
}

/// Rustdoc and Clippy share rustc with their installed toolchain.
fn compiler_for_driver(executable: PathBuf) -> (PathBuf, Option<&'static str>) {
    let cfg = match executable.file_stem().and_then(OsStr::to_str) {
        Some("rustdoc") => Some("doc"),
        Some("clippy-driver") => Some("clippy"),
        _ => None,
    };
    let compiler = if cfg.is_some() {
        executable.with_file_name(format!("rustc{}", env::consts::EXE_SUFFIX))
    } else {
        executable
    };
    (compiler, cfg)
}

/// Keep resolution flags, not the caller's input, output, lint, or linking flags.
fn resolution_arguments(arguments: &[OsString]) -> Result<Vec<OsString>, String> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        let text = argument
            .to_str()
            .ok_or("who: path() cannot capture a non-Unicode compiler argument")?;
        let (option, attached) = text
            .split_once('=')
            .map_or((text, None), |(key, value)| (key, Some(value)));
        if text == "--test" {
            result.extend([OsString::from("--cfg"), OsString::from("test")]);
        } else if text == "-O" {
            result.extend([OsString::from("-C"), OsString::from("opt-level=3")]);
        } else if matches!(
            option,
            "--edition"
                | "--target"
                | "--sysroot"
                | "--cfg"
                | "--check-cfg"
                | "--extern"
                | "--codegen"
        ) {
            let value = option_value(arguments, &mut index, attached, option)?;
            let option = if option == "--codegen" { "-C" } else { option };
            if (option != "--extern" || is_sysroot_extern(&value))
                && (option != "-C" || relevant_codegen(option, &value))
            {
                result.push(option.into());
                result.push(value);
            }
        } else if text == "-C"
            || text.starts_with("-C")
            || text == "-L"
            || text.starts_with("-L")
            || text == "-Z"
            || text.starts_with("-Z")
        {
            let option = &text[..2];
            let attached = (text.len() > 2).then(|| text[2..].trim_start_matches('='));
            let value = option_value(arguments, &mut index, attached, option)?;
            if option == "-L" || relevant_codegen(option, &value) {
                result.push(option.into());
                result.push(value);
            }
        }
        index += 1;
    }
    Ok(result)
}

/// Read either an attached option value or the following argument.
fn option_value(
    arguments: &[OsString],
    index: &mut usize,
    attached: Option<&str>,
    option: &str,
) -> Result<OsString, String> {
    match attached {
        Some(value) => Ok(value.into()),
        None => {
            *index += 1;
            arguments
                .get(*index)
                .cloned()
                .ok_or_else(|| format!("who: missing value for rustc {option}"))
        }
    }
}

/// Include sysroot overrides used by Cargo's build-std support.
fn is_sysroot_extern(value: &OsStr) -> bool {
    let value = value.to_string_lossy();
    let name = value
        .split('=')
        .next()
        .unwrap_or("")
        .rsplit(':')
        .next()
        .unwrap_or("");
    matches!(name, "std" | "core" | "alloc" | "proc_macro")
}

/// Preserve options that can alter cfgs or sysroot API availability.
fn relevant_codegen(option: &str, value: &OsStr) -> bool {
    let key = value.to_str().unwrap_or("").split('=').next().unwrap_or("");
    match option {
        "-C" => matches!(
            key,
            "target-cpu"
                | "target-feature"
                | "panic"
                | "opt-level"
                | "debug-assertions"
                | "overflow-checks"
        ),
        "-Z" => matches!(
            key,
            "crate-attr" | "allow-features" | "unstable-options" | "force-unstable-if-unmarked"
        ),
        _ => false,
    }
}

/// Expand rustc's newline-separated response arguments before filtering.
fn expand_response_files(arguments: Vec<OsString>, depth: usize) -> Result<Vec<OsString>, String> {
    if depth > 16 {
        return Err("who: rustc response files are nested too deeply".into());
    }
    let mut result = Vec::new();
    for argument in arguments {
        if let Some(path) = argument.to_str().and_then(|value| value.strip_prefix('@')) {
            let content = fs::read_to_string(path).map_err(|error| {
                format!("who: cannot read rustc response file `{path}`: {error}")
            })?;
            result.extend(expand_response_files(
                content.lines().map(OsString::from).collect(),
                depth + 1,
            )?);
        } else {
            result.push(argument);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_context_and_removes_build_outputs_and_dependencies() {
        let arguments = [
            "src/main.rs",
            "--crate-name",
            "caller",
            "--edition=2021",
            "--target",
            "wasm32-unknown-unknown",
            "--sysroot=/toolchain",
            "--cfg",
            "feature=\"foo\"",
            "--check-cfg=cfg(foo)",
            "-Ctarget-feature=+simd128",
            "-C",
            "panic=abort",
            "-Cmetadata=caller",
            "-Cincremental=/caller",
            "--extern",
            "tokio=/deps/tokio.rmeta",
            "--extern=noprelude:core=/deps/core.rmeta",
            "-Ldependency=/deps",
            "--emit=link",
            "--out-dir",
            "/caller",
            "-Dwarnings",
            "-Zcrate-attr=feature(test)",
        ]
        .map(OsString::from);
        let filtered = resolution_arguments(&arguments).unwrap();
        let expected = [
            "--edition",
            "2021",
            "--target",
            "wasm32-unknown-unknown",
            "--sysroot",
            "/toolchain",
            "--cfg",
            "feature=\"foo\"",
            "--check-cfg",
            "cfg(foo)",
            "-C",
            "target-feature=+simd128",
            "-C",
            "panic=abort",
            "--extern",
            "noprelude:core=/deps/core.rmeta",
            "-L",
            "dependency=/deps",
            "-Z",
            "crate-attr=feature(test)",
        ]
        .map(OsString::from);
        assert_eq!(filtered, expected);
        assert!(resolution_arguments(&["--target".into()]).is_err());
    }

    #[test]
    fn expands_response_arguments() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("args");
        fs::write(&path, "--edition=2021\n--cfg\nfeature=\"foo\"\n").unwrap();
        let arguments =
            expand_response_files(vec![format!("@{}", path.display()).into()], 0).unwrap();
        assert_eq!(
            arguments,
            ["--edition=2021", "--cfg", "feature=\"foo\""].map(OsString::from)
        );
    }

    #[test]
    fn compiler_drivers_keep_their_toolchain_and_implicit_cfg() {
        for (driver, cfg) in [
            ("rustdoc", Some("doc")),
            ("clippy-driver", Some("clippy")),
            ("rustc", None),
        ] {
            let executable = PathBuf::from(format!(
                "/toolchain/bin/{driver}{}",
                env::consts::EXE_SUFFIX
            ));
            let (compiler, actual_cfg) = compiler_for_driver(executable);
            assert_eq!(
                compiler,
                PathBuf::from(format!("/toolchain/bin/rustc{}", env::consts::EXE_SUFFIX))
            );
            assert_eq!(actual_cfg, cfg);
        }
    }

    #[test]
    fn preserves_implicit_cfgs_and_long_codegen_options() {
        let arguments = [
            "--test",
            "-O",
            "--codegen",
            "debug-assertions=no",
            "--codegen=target-cpu=native",
            "--codegen=incremental=/output",
        ]
        .map(OsString::from);
        let expected = [
            "--cfg",
            "test",
            "-C",
            "opt-level=3",
            "-C",
            "debug-assertions=no",
            "-C",
            "target-cpu=native",
        ]
        .map(OsString::from);
        assert_eq!(resolution_arguments(&arguments).unwrap(), expected);
    }
}
