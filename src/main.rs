use std::env;

use toby_ssg::{create_note, init, serve::serve, ssg::render, validate_working_directory};

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
        "note" => {
            if args.len() == 3 {
                create_note(&args[2]);
            } else {
                create_note("new_note")
            }
        }
        _ => panic!("Not a valid argument {}", { query }),
    }
}
