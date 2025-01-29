use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use askama::Template;
use glob::glob;
use markdown::{mdast::Node, to_html_with_options, to_mdast, Constructs, Options, ParseOptions};

use crate::{validate_working_directory, AppError};

struct Matter {
    title: String,
    publish: bool,
    tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate {
    title: String,
    content: String,
}

// basic handler that responds with a static string[]
#[axum::debug_handler]
pub async fn render() -> Result<(), AppError> {
    validate_working_directory();
    render_css()?;
    render_html()?;
    Ok(())
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

fn render_css() -> anyhow::Result<()> {
    let mut file = File::create("output/static/style.css")?;
    let css = grass::from_path("templates/style.scss", &grass::Options::default())?;
    file.write_all(css.as_bytes())?;
    Ok(())
}

fn render_html() -> anyhow::Result<()> {
    let options = markdown_options();
    for entry in glob("content/**/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // Reading the each markdown file and verifying if its valid to render it.
                let mut input = File::open(&path)?;
                let mut content = String::new();

                input.read_to_string(&mut content)?;
                let frontmatter = parse_frontmatter(&content);
                if frontmatter.is_none() { break }

                // Frontmatter
                let frontmatter = frontmatter.unwrap();
                if !frontmatter.publish {
                    break;
                }
                println!("{:#?}", frontmatter.tags);

                let (html_file_path, prefix) = html_path(&path);

                if fs::exists(&prefix).is_ok_and(|x| !x) {
                    fs::create_dir_all(prefix).unwrap();
                }

                content = to_html_with_options(&content, &options).unwrap();

                let mut output = File::create(html_file_path)?;
                let template = BaseTemplate {
                    title: frontmatter.title,
                    content,
                };
                output.write_all(template.render()?.as_bytes())?;
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}

fn html_path(path: &Path) -> (PathBuf, PathBuf) {
    let mut html_file_path = PathBuf::from_iter(path.components().skip(1));
    html_file_path.set_extension("html");
    let html_file_path = Path::new("output").join(html_file_path);
    let prefix = html_file_path.parent().unwrap().to_path_buf();

    (html_file_path, prefix)
}
