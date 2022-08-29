//! FastCGI server that takes the FILE parameter to a Markdown file and renders it to HTML.

use std::{io::Write, net::TcpListener};

use pulldown_cmark::{html, CowStr, Event, HeadingLevel, Options, Parser, Tag};

// If you're using this yourself, you'll probably want to change this :)
static HTML_PREFIX: &'static str = r#"Content-Type: text/html


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
        <h2 style="text-align: center;"><a href="/">soupy.me</a></h1>
        <hr />
"#;

static HTML_SUFFIX: &'static str = r#"
    </body>
</html>"#;

fn main() {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let listener = TcpListener::bind("127.0.0.1:9000").unwrap();
    fastcgi::run_tcp(
        move |mut req| {
            let file = req.param("FILE").unwrap();
            let path = std::path::Path::new(&file);
            let file_contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(e) => {
                    println!("{}, {:?}", e, e.kind());
                    // if (e.kind() == ErrorKind::NotFound) {
                    write!(&mut req.stdout(), "Status: 404 Not Found\r\n\r\n").unwrap();
                    write!(&mut req.stderr(), "Status: 404 Not Found\r\n\r\n").unwrap();
                    req.exit(404);
                    return;
                }
            };

            let mut heading_level: Option<HeadingLevel> = None;
            let parser = Parser::new_ext(&file_contents, options).filter_map(|event| match event {
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

            write!(&mut req.stdout(), "{}", HTML_PREFIX).unwrap();
            match html::write_html(&mut req.stdout(), parser) {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
            write!(&mut req.stdout(), "{}", HTML_SUFFIX).unwrap();
        },
        &listener,
    );
}
