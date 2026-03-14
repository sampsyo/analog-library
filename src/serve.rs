use crate::core::{Context, Error};
use crate::view;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::{
        StatusCode,
        header::{CONTENT_TYPE, HeaderMap, HeaderValue},
    },
    response::{IntoResponse, Response},
    routing::get,
};

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        dbg!(&self);
        match self {
            Error::NotFound(doi) => (StatusCode::NOT_FOUND, view::not_found_page(&doi)),
            Error::Parse(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::des_error_page(err.to_string()),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::other_error_page(self.to_string()),
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
    let paper_json = ctx.fetch_doi_json(&doi).await?;
    match query.format.as_deref() {
        Some("json") => Ok(json_resp(paper_json.as_ref())),
        _ => {
            let paper = serde_json::from_slice(paper_json.as_ref())?;
            Ok(ctx.render_paper(paper).await?.into_response())
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
