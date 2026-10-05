fn main() {
    who::error!(path(crate::foo::Bar).exists(), "local crate");
    who::error!(path(self::foo::Bar).exists(), "local module");
    who::error!(path(super::foo::Bar).exists(), "parent module");
    who::error!(path(semver::Version).exists(), "external crate not implemented");
}
