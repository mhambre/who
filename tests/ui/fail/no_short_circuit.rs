fn main() {
    who::warn!(rustc().compare("<1") && dependency("who-nonexistent-package").compare("*"), "validate both operands");
}
