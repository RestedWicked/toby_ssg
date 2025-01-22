use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use askama::Template;
use axum::{body::Body, extract::Request, http::{Response, StatusCode, Uri}, routing::get, Router};
use glob::glob;
use markdown::to_html;
use toby_ssg::{init_dir, init_file, is_directory_empty, AppError};
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate {
    title: String,
    content: String,
}

#[tokio::main]
async fn main() {
    init();
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/render", get(render))
        .fallback_service( get(handler))
        .layer(LiveReloadLayer::new());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(uri: Uri) -> Result<Response<Body>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => get_static_file(uri_html).await,
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    match ServeDir::new("./output").oneshot(req).await {
        Ok(res) => Ok(res.map(Body::new)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}

fn init() {
    let md = fs::metadata(".").unwrap();
    let permissions = md.permissions();
    if permissions.readonly() {
        panic!("No write permissions");
    }

    if fs::exists("toby.toml").is_ok_and(|x| !x) {
        if !is_directory_empty(".") {
            panic!("Directory is not empty");
        }
        let toby = include_bytes!("../toby.toml");
        init_file("toby.toml", toby);
    }

    init_dir("templates");
    init_dir("content");
    init_dir("output");
    init_dir("output/static");

    let base_template = include_bytes!("../templates/base.html");
    let style = include_bytes!("../templates/style.scss");
    let index_md = include_bytes!("../content/index.md");

    init_file("templates/base.html", base_template);
    init_file("templates/style.scss", style);
    init_file("content/index.md", index_md);
}

// basic handler that responds with a static string
async fn render() -> Result<(), AppError> {
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
