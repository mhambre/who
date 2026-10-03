fn main() {
    who::warn!(dependency("who-nonexistent-package").matches("*"), "missing package");
}
