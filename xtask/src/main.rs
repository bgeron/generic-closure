#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use comrak::{Arena, Options, format_commonmark, nodes::NodeValue, parse_document};
use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

const GENERATED_NOTICE: &str =
    "<!-- Generated from src/crate.md by `cargo xtask readme`. Do not edit directly. -->\n\n";

#[derive(Parser)]
#[command(bin_name = "cargo xtask")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate README.md from the crate documentation.
    Readme {
        /// Check that README.md is current without modifying it.
        #[arg(long)]
        check: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Command::Readme { check } => update_readme(check),
    }
}

fn update_readme(check: bool) -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let source_path = root.join("src/crate.md");
    let readme_path = root.join("README.md");
    let source = fs::read_to_string(&source_path)?;
    let generated = render_readme(&source)?;

    if check {
        let current = fs::read_to_string(&readme_path)?;
        if current != generated {
            return Err(
                io::Error::other("README.md is out of date; run `cargo xtask readme`").into(),
            );
        }
    } else if fs::read_to_string(&readme_path).ok().as_deref() != Some(&generated) {
        fs::write(readme_path, generated)?;
    }

    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be directly inside the workspace root")
        .to_owned()
}

fn render_readme(source: &str) -> Result<String, fmt::Error> {
    let arena = Arena::new();
    let mut options = Options::default();
    options.render.prefer_fenced = true;
    options.render.experimental_minimize_commonmark = true;

    let document = parse_document(&arena, source, &options);
    for node in document.descendants() {
        if let NodeValue::CodeBlock(code_block) = &mut node.data_mut().value {
            if is_rust_code_block(&code_block.info) {
                code_block.literal = without_rustdoc_hidden_lines(&code_block.literal);
            }
        }
    }

    let mut output = String::with_capacity(GENERATED_NOTICE.len() + source.len());
    output.push_str(GENERATED_NOTICE);
    format_commonmark(document, &options, &mut output)?;
    Ok(output)
}

fn is_rust_code_block(info: &str) -> bool {
    info.split(|character: char| character == ',' || character.is_whitespace())
        .any(|word| word == "rust")
}

fn without_rustdoc_hidden_lines(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        let content = content.strip_suffix('\r').unwrap_or(content);
        let code = content.trim_start_matches([' ', '\t']);
        let indentation = content.len() - code.len();

        if code == "#" || code.starts_with("# ") {
            continue;
        }

        if code.starts_with("##") {
            output.push_str(&line[..indentation]);
            output.push_str(&line[indentation + 1..]);
        } else {
            output.push_str(line);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command, GENERATED_NOTICE, render_readme, without_rustdoc_hidden_lines};
    use clap::Parser;

    #[test]
    fn clap_parses_the_readme_subcommand() {
        let cli = Cli::try_parse_from(["cargo xtask", "readme", "--check"]).unwrap();
        assert!(matches!(cli.command, Command::Readme { check: true }));
    }

    #[test]
    fn parses_markdown_and_transforms_only_rust_code_blocks() {
        let source = concat!(
            "# Heading\n\n",
            "```rust\n",
            "# use example::Thing;\n",
            "#\n",
            "let visible = 42;\n",
            "##[allow(dead_code)]\n",
            "```\n\n",
            "```sh\n",
            "# Keep this shell comment.\n",
            "```\n",
        );
        let expected = format!(
            "{GENERATED_NOTICE}{}",
            concat!(
                "# Heading\n\n",
                "```rust\n",
                "let visible = 42;\n",
                "#[allow(dead_code)]\n",
                "```\n\n",
                "```sh\n",
                "# Keep this shell comment.\n",
                "```\n",
            )
        );

        assert_eq!(render_readme(source).unwrap(), expected);
    }

    #[test]
    fn recognizes_rustdoc_fence_attributes_and_indented_hidden_lines() {
        let source = "```rust,no_run\n    # let hidden = 1;\n    visible();\n```\n";
        let rendered = render_readme(source).unwrap();

        assert!(!rendered.contains("hidden"));
        assert!(rendered.contains("    visible();"));
    }

    #[test]
    fn transforms_rustdoc_lines_independently_of_markdown_fences() {
        let source = "# hidden\nvisible();\n##[allow(dead_code)]\n";
        assert_eq!(
            without_rustdoc_hidden_lines(source),
            "visible();\n#[allow(dead_code)]\n"
        );
    }
}
