fn main() {
    who::warn!(dependency("who-nonexistent-package").compare("*"), "missing package");
}
