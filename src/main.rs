use std::env;

use toby_ssg::{init, serve::serve, ssg::render, validate_working_directory};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    if args.len() == 1 {
        validate_working_directory();
        return;
    }

    let query = &args[1];
    match query.as_str() {
        "init" => init(),
        "serve" => serve().await,
        "render" => render().await.expect("Could Not Render"),
        _ => panic!("Not a valid argument {}", { query }),
    }
}
