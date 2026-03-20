use crate::bib;
use crate::core::{ASSETS, join};
use crate::crossref::Paper;
use crate::jats;
use maud::{DOCTYPE, Markup, PreEscaped, html};

fn page(title: &str, main: Markup, head: Markup) -> Markup {
    #[cfg(debug_assertions)]
    let css = ASSETS.read("style.css").expect("asset must exist").unwrap();

    #[cfg(not(debug_assertions))]
    let css = ASSETS.get("style.css").expect("asset must exist");

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) };
                style { (PreEscaped(css)) };
                (head);
            }
        }
        body { main { (main) } }
    }
}

pub fn paper(paper: Paper, abstract_: Option<String>) -> Markup {
    let title = paper.title();

    // Try converting the abstract from JATS XML to HTML we can render. If this
    // fails, just pass through the XML as text.
    // TODO we should probably log the error.
    let abs = match &abstract_ {
        Some(j) => {
            let content = match jats::to_html(j) {
                Ok(h) => html! { (PreEscaped(h)) },
                Err(_) => html! { (j) },
            };
            html! {
                div.abstract { (content) }
            }
        }
        None => {
            html! { div.abstract.missing { "Data missing." } }
        }
    };

    let main = html! {
        nav {
            div.details {
                span.type {
                    ( paper.human_type() )
                }
                span.doi {
                    ( paper.doi )
                }
            }
            div.links {
                @if let Some(url) = paper.resource_url() {
                    a href=(url) { ( paper.domain().unwrap() ) }
                }
                @if let Some(url) = paper.pdf_url() {
                    a href=(url) { "PDF" }
                }
            }
        }
        h1 { (title) };
        span.label { "Authors:" } " "
        div.authors {
            span.author { (paper.author[0].name()) }
            @for author in &paper.author[1..] {
                ", "
                span.author { (author.name()) }
            }
        };
        span.label { "Published:" } " "
        div.published {
            @if paper.type_ == "journal-article" {
                (paper.container_title)
                @if let Some(vol) = &paper.volume {
                    (", volume ") (vol)
                }
                @if let Some(iss) = &paper.issue {
                    (", issue ") (iss)
                }
                @if let Some(page) = &paper.page {
                    (", pp. ")
                    (page)
                }
                (". ")
                (paper.published)
                (".")
            } @else if paper.type_ == "proceedings-article" {
                ("In ")
                (paper.event.as_deref().unwrap_or(""))
                (". ")
                (paper.published)
                (".")
            }
        }
        span.label { "Abstract:" } " "
        (abs)
        span.label { "BibTeX:" } " "
        pre.bibtex {
            (bib::Entry(&paper))
        }
    };

    let doi_url = format!("https://doi.org/{}", paper.doi);
    let authors = join(paper.author.iter().map(|a| a.name()), ", ");
    let head = html! {
        meta property="og:title" content=(title);
        meta property="og:url" content=(doi_url);
        @if let Some(abs) = &abstract_ && let Ok(abs) = jats::to_text(abs) {
            meta property="og:description" content=(abs);
        }
        meta property="og:type" content="article";
        meta property="article:author" content=(authors);
        meta property="article:published_time" content=(paper.published.iso());
    };

    page(&title, main, head)
}

pub fn home(host: &str) -> Markup {
    #[cfg(debug_assertions)]
    let home = ASSETS.read("home.html").expect("asset must exist").unwrap();

    #[cfg(not(debug_assertions))]
    let home = ASSETS.get("home.html").expect("asset must exist");

    let home = home.replace("__HOST__", host);
    let home = home.replace("__VERSION__", env!("CARGO_PKG_VERSION"));

    page(
        "Analog Library Premium Edition™",
        PreEscaped(home),
        html! {},
    )
}

pub fn doi_not_found(doi: &str) -> Markup {
    page(
        "404 Not Found",
        html! {
            h1 { "404 Not Found" }
            p {
                ("Analog Library does not have data for the DOI ")
                code { (doi) } (". ")
                ("It uses the ")
                a href="https://www.crossref.org" { "Crossref" }
                (" database, so only DOIs present there can be rendered.")
            }
        },
        html! {},
    )
}

pub fn route_not_found() -> Markup {
    page(
        "404 Not Found",
        html! {
            h1 { "404 Not Found" }
            p {
                ("No page with that URL exists. Remember to use ")
                code { ("/doi/") }
                (" before the DOI in the URL.")
            }
        },
        html! {},
    )
}

pub fn des_error(msg: String) -> Markup {
    page(
        "500 Could Not Parse API Response",
        html! {
            h1 { "500 Could Not Parse API Response" }
            p {
                ("Analog Library could not parse the response from the ")
                a href="https://www.crossref.org" { "Crossref" }
                " API. Here is the deserialization error:"
            }
            pre { (msg) }
            p {
                ("If you like, you can ")
                a href="https://codeberg.org/samps/analog-library/issues" {
                    ("file a bug")
                }
                (" about this DOI and we might be able to fix it. ")
                ("To see the raw JSON response, append ")
                code { ("?format=json") }
                " to the URL."
            }
        },
        html! {},
    )
}

pub fn other_error(msg: String) -> Markup {
    page(
        "500 Internal Server Error",
        html! {
            h1 { "500 Internal Server Error" }
            p {
                ("Analog Library encountered an internal error:")
            }
            pre { (msg) }
        },
        html! {},
    )
}
