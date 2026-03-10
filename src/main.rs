mod core;
mod crossref;
mod jats;
mod serve;
mod view;
mod webcache;

use core::Context;

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let ctx = Context::default();
    match args.subcommand().unwrap().as_deref() {
        None | Some("serve") => {
            serve::serve(ctx);
        }
        Some(cmd) => {
            eprintln!("unknown command {cmd}");
            eprintln!("available commands are: serve");
            std::process::exit(1);
        }
    }
}
