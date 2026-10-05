fn is_vendored_python_keyword(word: &str) -> bool {
    who::warn!(
        file("vendor/python-keywords.txt")
            .changed_from("sha256:60812d3ca89d1be613e875bb24ccccbcb4c57d255b91cba1ac194b5348cb1f7d"),
        "Review is_vendored_python_keyword; it was handwritten from vendor/python-keywords.txt"
    );

    matches!(word, "False" | "None" | "True" | "and" | "as" | "assert")
}

fn main() {
    let words = ["assert", "lambda", "True", "value"];
    let vendored = words
        .into_iter()
        .filter(|word| is_vendored_python_keyword(word))
        .collect::<Vec<_>>();

    println!("Vendored subset matches: {}", vendored.join(", "));
}
