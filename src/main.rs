use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use basset::assets;

assets!(TEMPLATES, "templates", ["paper.html", "style.css"]);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DOIData {
    title: String,
}

enum MyError {
    DOIFetchFailed,
    RenderFailed,
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::DOIFetchFailed => "failed to retrieve data from doi.org",
            MyError::RenderFailed => "page rendering failed",
        };

        // it's often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

async fn fetch_doi(doi: &str) -> Result<DOIData, MyError> {
    // TODO validate DOI
    let doi_url = format!("https://doi.org/{doi}");

    // TODO cache results
    let client = reqwest::Client::new();
    let data = client
        .get(doi_url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| MyError::DOIFetchFailed)?
        .json()
        .await
        .map_err(|_| MyError::DOIFetchFailed)?;

    Ok(data)
}

async fn paper(State(ctx): State<Context>, Path(doi): Path<String>) -> Result<Response, MyError> {
    let doi_data = fetch_doi(&doi).await?;

    // Render the page.
    let tmpl = ctx
        .tmpls
        .get_template("paper.html")
        .expect("template must exist");
    let body = tmpl.render(doi_data).map_err(|_| MyError::RenderFailed);

    Ok(Html(body).into_response())
}

#[derive(Clone)]
struct Context {
    tmpls: minijinja::Environment<'static>,
}

fn templates() -> minijinja::Environment<'static> {
    let mut env = minijinja::Environment::new();

    // Register embedded templates, which are available in release mode.
    #[cfg(not(debug_assertions))]
    for (name, source) in TEMPLATES.contents() {
        env.add_template(name, source)
            .expect("error in embedded template");
    }

    // In debug mode only, load templates directly from the filesystem.
    #[cfg(debug_assertions)]
    for (name, source) in TEMPLATES.read_all() {
        env.add_template_owned(name, source.expect("error reading template"))
            .expect("error in loaded template");
    }

    env
}

#[tokio::main]
async fn main() {
    let ctx = Context { tmpls: templates() };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doi/{*doi}", get(paper))
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
