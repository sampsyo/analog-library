mod bib;
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
    #[error("could not decode JATS XML: {0}")]
    Jats(#[from] jats::Error),
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
            let json = ctx.fetch_doi_json(&doi).await?;
            std::io::stdout().write_all(json.as_ref())?;
        }
        Some("html") => {
            let doi: String = args.free_from_str()?;
            let paper = ctx.fetch_doi(&doi).await?;
            let html = ctx.render_paper(paper).await?;
            println!("{}", html.into_string());
        }
        Some("bib") => {
            let doi: String = args.free_from_str()?;
            let paper = ctx.fetch_doi(&doi).await?;
            println!("{}", bib::Entry(&paper));
        }
        Some("abs") => {
            let doi: String = args.free_from_str()?;
            let paper = ctx.fetch_doi(&doi).await?;
            let abs = ctx.get_abstract(&paper).await?;
            if let Some(abs) = abs {
                let abs = jats::to_text(&abs)?;
                println!("{abs}");
            }
        }
        Some("cache") => {
            ctx.dump_cache()?;
        }
        Some(cmd) => {
            eprintln!("unknown command {cmd}");
            eprintln!("available commands are: serve, json, html, bib, abs, cache");
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
