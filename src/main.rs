mod core;
mod crossref;
mod jats;
mod serve;
mod view;
mod webcache;

use core::Context;
use std::io::Write;
use std::process::ExitCode;

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error("could not parse arguments: {0}")]
    Args(#[from] pico_args::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Core(#[from] core::Error),
}

async fn run() -> Result<(), MainError> {
    let mut args = pico_args::Arguments::from_env();
    let ctx = Context::default();
    match args.subcommand()?.as_deref() {
        None | Some("serve") => {
            serve::serve(ctx).await;
        }
        Some("json") => {
            let doi: String = args.free_from_str()?;
            let json = core::fetch_doi_json(&ctx, &doi).await?;
            std::io::stdout().write_all(json.as_ref())?;
        }
        Some("html") => {
            let doi: String = args.free_from_str()?;
            let paper = core::fetch_doi(&ctx, &doi).await?;
            let html = core::render_paper(&ctx, paper).await?;
            println!("{}", html.into_string());
        }
        Some(cmd) => {
            eprintln!("unknown command {cmd}");
            eprintln!("available commands are: serve");
            std::process::exit(1);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}
