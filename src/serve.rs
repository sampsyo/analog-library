use crate::core::{Context, Error, fetch_doi, fetch_doi_json};
use crate::{crossref, view};
use axum::{
    Router,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
    routing::get,
};

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        dbg!(&self);
        let msg = self.to_string();
        let code = match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, msg).into_response()
    }
}

/// Find an abstract for this paper.
///
/// Some papers in the Crossref database have several "identical" entries, with
/// different DOIs and different sets of metadata. When a paper is missing an
/// abstract, it is often the case that other identical entries *do* have an
/// abstract. So we first try the abstract we already have and, if it's missing,
/// try all the identical entries to see if they have one we can use.
async fn get_abstract(ctx: &Context, paper: &crossref::Paper) -> Result<Option<String>, Error> {
    match &paper.abstract_ {
        Some(abs) => Ok(Some(abs.to_string())),
        None => {
            let mut out = None;
            for other_doi in paper.identical_dois() {
                // TODO Maybe try to suppress "not found" errors when fetching other_paper?
                let other_paper = fetch_doi(ctx, &other_doi).await?;
                if let Some(abstract_) = other_paper.abstract_ {
                    out = Some(abstract_.to_string());
                    break;
                }
            }
            Ok(out)
        }
    }
}

fn json_resp(json: &[u8]) -> Response {
    let buf = Vec::<u8>::from(json);
    (
        [(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )],
        buf,
    )
        .into_response()
}

async fn show_paper(
    State(ctx): State<Context>,
    headers: HeaderMap,
    Path(doi): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let paper_json = fetch_doi_json(&ctx, &doi).await?;
    match headers.get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"application/json") => Ok(json_resp(paper_json.as_ref())),
        _ => {
            let paper = serde_json::from_slice(paper_json.as_ref())?;
            let abstract_ = get_abstract(&ctx, &paper).await?;
            Ok(view::paper_page(paper, abstract_).into_response())
        }
    }
}

async fn show_home(headers: HeaderMap) -> maud::Markup {
    let host = match headers.get("Host") {
        Some(h) => h.to_str().unwrap_or("example.com"),
        None => "example.com",
    };
    view::home_page(host)
}

pub async fn serve(ctx: Context) {
    let app = Router::new()
        .route("/", get(show_home))
        .route("/doi/{*doi}", get(show_paper))
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    eprintln!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
