use clap::{Parser, Subcommand};
use toby_ssg::{create_note, init, serve::serve, ssg::render, validate_working_directory};

/// Toby SSG
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
#[clap(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a directory for usage with Toby SSG.
    Init,
    /// Serve prerendered site files.
    Serve,
    /// Render markdown files.
    Render,
    /// Create new markdown file.
    Note {
        /// File name for new note. Example: cat_notes
        name: Option<String>,
    },
    /// Validate working directory.
    Validate,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => init(),
        Some(Commands::Serve) => serve().await,
        Some(Commands::Render) => {
            let _ = render().await;
        }
        Some(Commands::Note { name }) => {
            if let Some(name) = name {
                create_note(&name)
            } else {
                create_note("new_note")
            }
        }
        Some(Commands::Validate) => validate_working_directory(),
        None => (),
    }
}
