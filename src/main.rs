//! FastCGI server that takes the FILE parameter to a Markdown file and renders it to HTML.

use std::{io::Write, net::TcpListener, path::PathBuf, fs, ffi::OsStr};
use structopt::{StructOpt, clap::Arg};
use pulldown_cmark::{HeadingLevel, Event, Tag, CowStr};

// If you're using this yourself, you'll probably want to change this :)
// TODO: move this to a template file
static DEFAULT_TEMPLATE: &'static str = r#"
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
            h1 {
                text-align: center;
                font-family: Tahoma, Verdana, Arial, sans-serif;
            }
        </style>
    </head>
    <body>
        $$CONTENT$$
    </body>
    </html>
"#;

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

    #[structopt(help = "Markdown files or directories containing .md files", name = "files or directories to process", required = true)]
    input_dirs_and_files: Vec<PathBuf>,
}

fn md_to_html(markdown: String, template: &String) -> String {
    let mut md_parser_options = pulldown_cmark::Options::empty();
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    md_parser_options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    let mut heading_level: Option<HeadingLevel> = None;
    let md_parser = pulldown_cmark::Parser::new_ext(&markdown, md_parser_options).filter_map(|event| match event {
        Event::Start(Tag::Heading(level, _, _)) => {
            heading_level = Some(level);
            None
        }
        Event::Text(text) => {
            if let Some(level) = heading_level {
                let anchor = text
                    .clone()
                    .into_string()
                    .trim()
                    .to_lowercase()
                    .replace(" ", "-");
                let tmp = Event::Html(CowStr::from(format!(
                    "<{} id=\"{}\">{}",
                    level, anchor, text
                )))
                .into();
                heading_level = None;
                return tmp;
            }
            Some(Event::Text(text))
        }
        _ => Some(event),
    });

    // could be more efficient but eh
    let mut content = String::new();
    pulldown_cmark::html::push_html(&mut content, md_parser);

    template.replace("$$CONTENT$$", &content)
}

fn process_path(path: PathBuf, template: &String) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in path.read_dir()? {
            process_path(entry?.path(), template);
        }
    } else if path.is_file() {
        if path.extension() == Some(OsStr::new("md")) {
            process_file(path, template);
        } else {
            eprintln!("Warning: ignoring non-Markdown file '{}'", path.display());
        }
    } else {
        eprintln!("Warning: ignoring non-file, non-directory '{}'", path.display());
    }

    Ok(())
}

// TODO: fix errors
// TODO: clean up, use results
// TODO: benchmark
// TODO: title directive support
// TODO: remove unneeded packages/imports

fn process_file(input_path: PathBuf, template: &String) -> std::io::Result<()> {
    let markdown_text = fs::read_to_string(&input_path)?;
    let mut output_path = input_path.clone();

    output_path.set_extension("html");
    if &output_path.to_string_lossy() == &input_path.to_string_lossy() {
        eprintln!("Warning: output for '{}' may overwrite the original file — ignoring", input_path.display());
        return Ok(());
    }

    fs::write(&output_path, md_to_html(markdown_text, template))?;
    println!("{}", output_path.display());
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Arguments::from_args();

    let template = if let Some(template_path) = args.template_path {
        let template_string = match fs::read_to_string(&template_path) {
            Err(e) => {
                eprintln!("Error: Couldn't read template file '{}': {}", template_path.display(), e);
                std::process::exit(1);
            }
            Ok(str) => str,
        };

        if !template_string.contains("$$CONTENT$$") {
            eprintln!("Error: Template file '{}' does not include '$$CONTENT$$' - exiting.", template_path.display());
            std::process::exit(1);
        }

        template_string
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    for path in args.input_dirs_and_files {
        process_path(path, &template);
    }


            // write!(&mut req.stdout(), "{}", HTML_PREFIX).unwrap();
            // write!(&mut req.stdout(), "{}", HTML_SUFFIX).unwrap();\
    Ok(())
}
