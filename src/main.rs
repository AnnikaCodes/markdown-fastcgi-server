//! mdtransform - small tool that converts Markdown to simple HTML

use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag};
use regex::Regex;
use std::{ffi::OsStr, fs, path::PathBuf};
use structopt::StructOpt;

// TODO: support <title> prefix
static DEFAULT_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
    <head>
        <style>
            body {
                width: max(50em, min(500px, 95vw));
                margin: 0 auto;
                font-family: sans-serif;
                font-size: 1.1rem;
            }
        </style>
    </head>
    <body>
        $$CONTENT$$
    </body>
    </html>
"#;

lazy_static::lazy_static! {
    static ref HEADING_REGEX: Regex = Regex::new("[^a-z-]*").expect("couldn't compile regex");
    static ref HTML_EXTENSION: Option<&'static OsStr> = Some(OsStr::new("html"));
    static ref MARKDOWN_EXTENSION: Option<&'static OsStr> = Some(OsStr::new("md"));
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mdtransform",
    about = "transforms Markdown files into HTML for a website"
)]
struct Arguments {
    #[structopt(
        short = "-t",
        long = "--template",
        help = "path to a HTML template file; should include the string '$$CONTENT$$' which will be replaced with the HTML body"
    )]
    template_path: Option<PathBuf>,

    #[structopt(
        help = "Markdown files or directories containing .md files",
        name = "files or directories to process",
        required = true
    )]
    input_dirs_and_files: Vec<PathBuf>,
}

fn md_to_html(markdown: String, template: &str) -> String {
    let mut md_parser_options = pulldown_cmark::Options::empty();
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    let mut heading_level: Option<HeadingLevel> = None;
    let md_parser = pulldown_cmark::Parser::new_ext(&markdown, md_parser_options).filter_map(
        |event| match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                heading_level = Some(level);
                None
            }
            Event::Text(text) => {
                if let Some(level) = heading_level {
                    // Handle title directive
                    if level == HeadingLevel::H1 {
                        if let Some(title) = text.strip_prefix("TITLE: ") {
                            let html = [
                                "<title>",
                                title,
                                "</title>\n",
                                "<center>\n",
                                "\t<h1 style='margin-bottom: 0px; font-size: 2.5rem;'>",
                                title,
                                "</h1>\n",
                                "\t<hr />\n",
                                "</center>",
                            ]
                            .join("");

                            heading_level = None;

                            // We don't need a heading for a title, do we?
                            return Some(Event::Html(CowStr::from(html)));
                        }
                    }

                    let anchor = text
                        .clone()
                        .into_string()
                        .trim()
                        .to_lowercase()
                        .replace(' ', "-");
                    let anchor = HEADING_REGEX.replace_all(&anchor, "");
                    let tmp = Event::Html(CowStr::from(format!(
                        "<{level} id=\"{anchor}\">{text}"
                    )))
                    .into();
                    heading_level = None;
                    return tmp;
                }
                Some(Event::Text(text))
            }
            _ => Some(event),
        },
    );

    // could be more efficient but eh
    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, md_parser);

    template.replace("$$CONTENT$$", &content)
}

fn process_path(path: PathBuf, template: &String) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in path.read_dir()? {
            process_path(entry?.path(), template)?;
        }
    } else if path.is_file() {
        let ext = path.extension();
        if ext == *MARKDOWN_EXTENSION {
            process_file(path, template)?;
        } else if ext != *HTML_EXTENSION {
            eprintln!(
                "Warning: ignoring non-Markdown, non-HTML file '{}'",
                path.display()
            );
        }
    } else {
        eprintln!(
            "Warning: ignoring non-file, non-directory '{}'",
            path.display()
        );
    }

    Ok(())
}

fn process_file(input_path: PathBuf, template: &str) -> std::io::Result<()> {
    let markdown_text = fs::read_to_string(&input_path)?;
    let mut output_path = input_path.clone();

    output_path.set_extension("html");
    if output_path.to_string_lossy() == input_path.to_string_lossy() {
        eprintln!(
            "Warning: output for '{}' may overwrite the original file — ignoring",
            input_path.display()
        );
        return Ok(());
    }

    fs::write(&output_path, md_to_html(markdown_text, template))?;
    println!("=> {}", output_path.display());
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Arguments::from_args();

    let template = if let Some(template_path) = args.template_path {
        let template_string = match fs::read_to_string(&template_path) {
            Err(e) => {
                eprintln!(
                    "Error: Couldn't read template file '{}': {}",
                    template_path.display(),
                    e
                );
                std::process::exit(1);
            }
            Ok(str) => str,
        };

        if !template_string.contains("$$CONTENT$$") {
            eprintln!(
                "Error: Template file '{}' does not include '$$CONTENT$$' - exiting.",
                template_path.display()
            );
            std::process::exit(1);
        }

        template_string
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    for path in args.input_dirs_and_files {
        process_path(path, &template)?;
    }

    Ok(())
}
