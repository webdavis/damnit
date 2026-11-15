use super::Cli;

const EVERY_HELP_TEXT: &str = include_str!("help_text.txt");

fn every_help_text(command: &clap::Command, path: &str, out: &mut String) {
    let path = format!("{path} {}", command.get_name()).trim().to_string();
    let texts = [
        ("about", command.get_about()),
        ("long_about", command.get_long_about()),
        ("after_help", command.get_after_help()),
    ];
    for (key, text) in texts {
        if let Some(text) = text {
            out.push_str(&format!("{path}: {key} = {text}\n"));
        }
    }
    for arg in command.get_arguments() {
        let texts = [("help", arg.get_help()), ("long_help", arg.get_long_help())];
        for (key, text) in texts {
            if let Some(text) = text {
                out.push_str(&format!("{path} {}: {key} = {text}\n", arg.get_id()));
            }
        }
    }
    for sub in command.get_subcommands() {
        every_help_text(sub, &path, out);
    }
}

#[test]
fn every_command_and_flag_keeps_its_help_text() {
    use clap::CommandFactory;
    let mut texts = String::new();
    every_help_text(&Cli::command(), "", &mut texts);
    assert_eq!(texts, EVERY_HELP_TEXT);
}

fn every_help_page(command: &clap::Command, out: &mut Vec<(String, String)>) {
    let name = command.get_bin_name().unwrap_or_default().to_string();
    out.push((name, command.clone().render_help().to_string()));
    for sub in command.get_subcommands() {
        every_help_page(sub, out);
    }
}

#[test]
fn no_help_line_passes_100_columns() {
    use clap::CommandFactory;
    let mut root = Cli::command();
    root.build();
    let mut pages = Vec::new();
    every_help_page(&root, &mut pages);
    assert!(pages.iter().any(|(name, _)| name == "dam remote add"));
    for (name, page) in pages {
        for line in page.lines() {
            assert!(line.chars().count() <= 100, "{name}: {line}");
        }
    }
}
