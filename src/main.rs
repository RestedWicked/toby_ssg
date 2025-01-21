use std::{
    fs::File,
    io::{Read, Write},
};

use askama::Template;
use axum::{routing::get, Router};
use markdown::to_html;
use toby_ssg::AppError;
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
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/render", get(render))
        .fallback_service(ServeDir::new("output/"))
        .layer(LiveReloadLayer::new());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
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
    let mut input = File::open("content/index.md")?;
    let mut content = String::new();

    input.read_to_string(&mut content)?;
    content = to_html(&content);

    let mut output = File::create("output/index.html")?;
    let template = BaseTemplate {
        title: "Hai!!!".into(),
        content,
    };
    output.write_all(template.render()?.as_bytes())?;
    Ok(())
}
