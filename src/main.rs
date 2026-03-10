mod core;
mod crossref;
mod jats;
mod serve;
mod view;
mod webcache;

use core::Context;

fn main() {
    let ctx = Context::default();
    serve::serve(ctx);
}
