pub struct Example {
    pub line: usize,
    pub holder: Option<String>,
    pub exit: i32,
    pub rule: Option<String>,
    pub script: String,
}

pub fn examples(markdown: &str) -> Vec<Example> {
    let mut found = Vec::new();
    let mut lines = markdown.lines().enumerate();
    while let Some((at, line)) = lines.next() {
        let Some(info) = line.strip_prefix("```sh example") else {
            continue;
        };
        let script: Vec<&str> = lines
            .by_ref()
            .map(|(_, line)| line)
            .take_while(|line| *line != "```")
            .collect();
        let mut example = Example {
            line: at + 1,
            holder: None,
            exit: 0,
            rule: None,
            script: script.join("\n"),
        };
        for word in info.split_whitespace() {
            match word.split_once('=') {
                Some(("as", name)) => example.holder = Some(name.to_string()),
                Some(("exit", code)) => example.exit = code.parse().unwrap(),
                Some(("rule", rule)) => example.rule = Some(rule.to_string()),
                _ => panic!("line {}: unknown example attribute {word}", at + 1),
            }
        }
        found.push(example);
    }
    found
}
