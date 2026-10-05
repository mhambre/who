use std::fs;
use std::path::Path;
use std::process::{Command, Output};

pub struct CargoProject {
    directory: tempfile::TempDir,
    name: String,
}

impl CargoProject {
    /// Create an isolated consumer with every optional who feature disabled.
    pub fn new(name: &str) -> Self {
        let project = Self {
            directory: tempfile::tempdir().unwrap(),
            name: name.into(),
        };
        fs::create_dir(project.root().join("src")).unwrap();
        project.features(&[]);
        project
    }

    pub fn root(&self) -> &Path {
        self.directory.path()
    }

    /// Replace only the consumer's feature selection, keeping its isolated workspace.
    pub fn features(&self, features: &[&str]) {
        let manifest = format!(
            "[package]\nname={:?}\nversion='0.1.0'\nedition='2021'\n[workspace]\n[dependencies]\nwho={{path={:?},default-features=false,features={features:?}}}\n",
            self.name,
            env!("CARGO_MANIFEST_DIR"),
        );
        fs::write(self.root().join("Cargo.toml"), manifest).unwrap();
    }

    /// Compile offline with a private target directory and no inherited Rust flags.
    pub fn check(&self) -> Output {
        self.run(&["check"])
    }

    /// Run a Cargo command with the same isolated environment as checks.
    pub fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO"))
            .args(arguments)
            .arg("--offline")
            .current_dir(self.root())
            .env("CARGO_TARGET_DIR", self.root().join("target"))
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .output()
            .unwrap()
    }
}
