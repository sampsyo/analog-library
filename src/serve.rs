use crate::core::{Context, Error, RSRC, Source};
use crate::view;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::{
        StatusCode,
        header::{self, CONTENT_TYPE, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use tracing::debug;

/// Common URL components that, because of automatic redirection utilities, can
/// _accidentally_ show up on DOIs.
const STRIP_PREFIX: &[&str] = &[
    "abs/",
    "pdf/",
    "doi/",
    "doi.org/",
    "book/",
    "fullHtml/",
    "epdf/",
    "proceedings/",
];

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotFound(doi) => (StatusCode::NOT_FOUND, view::doi_not_found(&doi)),
            Error::Parse(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::des_error(err.to_string()),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::other_error(self.to_string()),
            ),
        }
        .into_response()
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

#[derive(serde::Deserialize)]
struct PaperQuery {
    format: Option<String>,
}

async fn show_paper(
    State(ctx): State<Context>,
    Path(doi): Path<String>,
    query: Query<PaperQuery>,
) -> Result<impl IntoResponse, Error> {
    for prefix in STRIP_PREFIX {
        if let Some(suffix) = doi.strip_prefix(prefix) {
            debug!(from = doi, to = suffix, "redirecting");
            let path = format!("/doi/{}", suffix);
            return Ok(Redirect::permanent(&path).into_response());
        }
    }

    match query.format.as_deref() {
        Some("json") => {
            let paper_json = ctx.fetch_doi(&doi, Source::Crossref).await?;
            Ok(json_resp(paper_json.as_ref()))
        }
        _ => Ok(ctx.render_paper(&doi).await?.into_response()),
    }
}

fn get_host(headers: &HeaderMap) -> &str {
    match headers.get("Host") {
        Some(h) => h.to_str().unwrap_or("example.com"),
        None => "example.com",
    }
}

async fn show_home(headers: HeaderMap) -> maud::Markup {
    view::home(get_host(&headers))
}

async fn send_rsrc(headers: HeaderMap, Path(filename): Path<String>) -> Response {
    let filename = filename.as_str();
    match RSRC.iter().find(|(f, _)| *f == filename) {
        Some((_, mime_type)) => {
            let body = view::asset(filename, get_host(&headers));
            let headers = [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime_type.as_ref()),
            )];
            (headers, body).into_response()
        }
        None => view::route_not_found().into_response(),
    }
}

pub async fn serve(ctx: Context) {
    let app = Router::new()
        .route("/", get(show_home))
        .route("/rsrc/{filename}", get(send_rsrc))
        .route("/doi/{*doi}", get(show_paper))
        .fallback(async || view::route_not_found())
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8118").await.unwrap();
    eprintln!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
