fn main() {
    who::warn!(rustc().matches("<1") && dependency("who-nonexistent-package").matches("*"), "validate both operands");
}
