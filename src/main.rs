mod core;
mod crossref;
mod jats;
mod serve;
mod view;
mod webcache;

use core::Context;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();
    let ctx = Context::default();
    match args.subcommand()?.as_deref() {
        None | Some("serve") => {
            serve::serve(ctx).await;
        }
        Some("json") => {
            let doi: String = args.free_from_str()?;
            match core::fetch_doi_json(&ctx, &doi).await {
                Ok(json) => {
                    std::io::stdout().write_all(json.as_ref()).unwrap();
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        Some(cmd) => {
            eprintln!("unknown command {cmd}");
            eprintln!("available commands are: serve");
            std::process::exit(1);
        }
    }
    Ok(())
}
