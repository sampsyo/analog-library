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
#[serde(rename_all = "kebab-case")]
struct DOIData {
    title: String,
    author: Vec<DOIAuthor>,
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "abstract")]
    abstract_: String,
    publisher: String,
    #[serde(rename = "URL")]
    url: String,

    container_title: String,
    page: String,
    volume: String,
    issue: String,

    issued: DOIDate,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIDate {
    date_parts: Vec<(u32, u32, u32)>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIAuthor {
    #[serde(rename = "ORCID")]
    orcid: String,
    given: String,
    family: String,
    sequence: String,
    affiliation: Vec<DOIAffiliation>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DOIAffiliation {
    name: String,
}

enum MyError {
    Fetch,
    Parse(serde_json::Error),
    Render,
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::Fetch => "failed to retrieve data from doi.org".to_string(),
            MyError::Parse(e) => format!("could not parse doi.org response: {e}"),
            MyError::Render => "page rendering failed".to_string(),
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
    let body = client
        .get(doi_url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| MyError::Fetch)?
        .bytes()
        .await
        .map_err(|_| MyError::Fetch)?;

    serde_json::from_slice(&body).map_err(|e| MyError::Parse(e))
}

async fn paper(State(ctx): State<Context>, Path(doi): Path<String>) -> Result<Response, MyError> {
    let doi_data = fetch_doi(&doi).await?;

    // Render the page.
    let tmpl = ctx
        .tmpls
        .get_template("paper.html")
        .expect("template must exist");
    let body = tmpl.render(doi_data).map_err(|_| MyError::Render);

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
