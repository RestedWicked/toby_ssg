use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use askama::Template;
use glob::glob;
use markdown::to_html;

use crate::{validate_working_directory, AppError};

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate {
    title: String,
    content: String,
}

// basic handler that responds with a static string
pub async fn render() -> Result<(), AppError> {
    validate_working_directory();
    render_css()?;
    render_html()?;
    Ok(())
}

fn render_css() -> anyhow::Result<()> {
    let mut file = File::create("output/static/style.css")?;
    let css = grass::from_path("templates/style.scss", &grass::Options::default())?;
    file.write_all(css.as_bytes())?;
    Ok(())
}

fn render_html() -> anyhow::Result<()> {
    for entry in glob("content/**/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let mut out_path = PathBuf::from_iter(path.components().skip(1));
                out_path.set_extension("html");
                let out_path = Path::new("output").join(out_path);
                let prefix = out_path.parent().unwrap();

                println!("{:#?}", out_path.display());

                if fs::exists(prefix).is_ok_and(|x| !x) {
                    fs::create_dir_all(prefix).unwrap();
                }

                let mut input = File::open(path)?;
                let mut content = String::new();

                input.read_to_string(&mut content)?;
                content = to_html(&content);

                let mut output = File::create(out_path)?;
                let template = BaseTemplate {
                    title: "Hai!!!".into(),
                    content,
                };
                output.write_all(template.render()?.as_bytes())?;
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}
