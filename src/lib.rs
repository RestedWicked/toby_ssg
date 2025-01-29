mod cli;
pub mod serve;
pub mod ssg;

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

// We need to validate the current working directory.
// For a directory to be valid we need:
// - Write Permissions
// - toby.toml to exist or
// - an empty directory to populate
pub fn validate_working_directory() {
    validate_working_directory_init(false);
}

// wanted an empty argument for every function that wasn't init lmaoo
fn validate_working_directory_init(is_init: bool) {
    let md = fs::metadata(".").unwrap();
    let permissions = md.permissions();
    if permissions.readonly() {
        panic!("No write permissions");
    }

    if fs::exists("toby.toml").is_ok_and(|x| !x) {
        if !is_directory_empty(".") {
            panic!("Directory is not empty");
        }
        if !is_init {
            panic!("Toby SSG has not been initilized");
        }
    }
}

pub fn is_directory_empty(directory: &str) -> bool {
    let mut entries = fs::read_dir(directory).expect("Could not read directory");
    entries.next().is_none()
}

pub fn init_file(path: &str, copy: &[u8]) {
    if fs::exists(path).is_ok_and(|x| !x) {
        let mut file = File::create(path).unwrap();
        file.write_all(copy).unwrap();
    }
}

pub fn init_dir(path: &str) {
    if fs::exists(path).is_ok_and(|x| !x) {
        fs::create_dir(path).unwrap();
    }
}

pub fn init() {
    validate_working_directory_init(true);

    let toby = include_bytes!("../toby.toml");
    init_file("toby.toml", toby);

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

pub fn create_note(note_name: &str) {
    validate_working_directory();
    let note = include_bytes!("../content/template.md");

    let mut path = PathBuf::new();
    path.push("content");
    path.push(note_name);
    path.set_extension("md");

    let prefix = &path.parent().unwrap().to_path_buf();

    if fs::exists(prefix).is_ok_and(|x| !x) {
        fs::create_dir_all(prefix).unwrap();
    }

    let mut count = 1;
    loop {
        if fs::exists(&path).is_ok_and(|x| x) {
            let file = format!("{note_name}_{count}.md");
            path.set_file_name(file);
            count += 1;
        } else {
            break;
        }
    }
    let mut file = File::create(path).unwrap();
    file.write_all(note).unwrap();
}
