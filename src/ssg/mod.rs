use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use askama::Template;
use axum::http::StatusCode;
use glob::glob;
use markdown::{mdast::Node, to_html_with_options, to_mdast, Constructs, Options, ParseOptions};

use crate::validate_working_directory;

struct Matter {
    title: String,
    publish: bool,
    tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate {
    title: String,
    tags: Vec<String>,
    content: String,
    links: DirectoryTemplate,
}

#[derive(Template)]
#[template(path = "directory.html")]
struct DirectoryTemplate {
    links: Vec<String>,
    titles: Vec<String>,
}

// basic handler that responds with a static string[]
#[axum::debug_handler]
pub async fn render() -> StatusCode {
    validate_working_directory();
    render_css();
    render_html();
    StatusCode::NO_CONTENT
}

fn render_css() {
    let mut file = File::create("output/static/styles/style.css").expect("Could not create style.css");
    let css = grass::from_path("templates/style.scss", &grass::Options::default())
        .expect("Could not read style.scss");
    file.write_all(css.as_bytes())
        .expect("Could not write to css");
}

fn render_html() {
    let options = markdown_options();
    let (entries, frontmatter, html_file_path, prefixes) = validate_markdown();
    let mut titles: Vec<String> = Vec::new();

    for matter in &frontmatter {
        titles.push(matter.title.to_owned());
    }
    for i in 0..entries.len() {
        let mut input = File::open(&entries[i]).expect("Could not open markdown file");
        let mut content = String::new();

        input
            .read_to_string(&mut content)
            .expect("Could not read to string");

        if fs::exists(&prefixes[i]).is_ok_and(|x| !x) {
            fs::create_dir_all(prefixes[i].clone()).unwrap()
        }

        content = to_html_with_options(&content, &options).unwrap();

        let mut output =
            File::create(html_file_path[i].clone()).expect("Could not create html file");
        let mut links: Vec<String> = Vec::new();
        html_file_path.clone().into_iter().for_each(|link| {
            let mut link = link.strip_prefix("output").unwrap().to_path_buf();
            link.set_extension("");
            if link.file_name() == Some(OsStr::new("index")) {
                link.set_file_name("");
            }
            links.push(link.to_str().unwrap().to_string());
        });
        let dir_template = DirectoryTemplate {
            links,
            titles: titles.clone(),
        };

        let template = BaseTemplate {
            title: frontmatter[i].title.clone(),
            tags: frontmatter[i].tags.clone(),
            content,
            links: dir_template,
        };
        output
            .write_all(
                template
                    .render()
                    .expect("Could not render template")
                    .as_bytes(),
            )
            .expect("Could not write template");
    }
}

fn validate_markdown() -> (Vec<PathBuf>, Vec<Matter>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut valid_entries: Vec<PathBuf> = Vec::new();
    let mut valid_frontmatter: Vec<Matter> = Vec::new();
    let mut valid_links: Vec<PathBuf> = Vec::new();
    let mut valid_prefixes: Vec<PathBuf> = Vec::new();

    for entry in glob("content/**/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // Reading the each markdown file and verifying if its valid to render it.
                let mut input = File::open(&path).expect("Could not open markdown file");
                let mut content = String::new();

                input
                    .read_to_string(&mut content)
                    .expect("Could not read to string");
                if let Some(frontmatter) = parse_frontmatter(&content) {
                    if !frontmatter.publish {
                        continue;
                    }
                    let (link, prefix) = html_path(&path);
                    valid_entries.push(path);
                    valid_frontmatter.push(frontmatter);
                    valid_links.push(link);
                    valid_prefixes.push(prefix);
                } else {
                    continue;
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    (
        valid_entries,
        valid_frontmatter,
        valid_links,
        valid_prefixes,
    )
}

fn parse_options() -> ParseOptions {
    let constructs = Constructs {
        frontmatter: true,
        ..Constructs::gfm()
    };

    ParseOptions {
        constructs,
        ..ParseOptions::gfm()
    }
}
fn markdown_options() -> Options {
    Options {
        parse: parse_options(),
        ..Options::gfm()
    }
}

fn parse_frontmatter(content: &str) -> Option<Matter> {
    let binding = to_mdast(content, &parse_options()).unwrap();
    let tree = binding.children().unwrap();
    for child in tree {
        if let Node::Yaml(yaml) = child {
            let yaml_contents: serde_yml::Value =
                serde_yml::from_str(&yaml.value.to_string()).unwrap();

            let mut title = String::new();
            let title_val = &yaml_contents["title"];
            if title_val.is_string() {
                title = title_val.as_str().unwrap().to_string();
            }

            let mut publish: bool = false;
            let publish_val = &yaml_contents["publish"];
            if publish_val.is_bool() {
                publish = publish_val.as_bool().unwrap();
            }

            let tags_value = yaml_contents["tags"].as_sequence().unwrap();
            let mut tags: Vec<String> = Vec::new();
            for tag in tags_value {
                tags.push(tag.as_str().unwrap().to_string());
            }
            let frontmatter = Matter {
                title,
                publish,
                tags,
            };
            return Some(frontmatter);
        }
    }
    None
}

fn html_path(path: &Path) -> (PathBuf, PathBuf) {
    let mut html_file_path = PathBuf::from_iter(path.components().skip(1));
    html_file_path.set_extension("html");
    let html_file_path = Path::new("output").join(html_file_path);
    let prefix = html_file_path.parent().unwrap().to_path_buf();

    (html_file_path, prefix)
}
