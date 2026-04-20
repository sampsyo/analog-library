use crate::bib;
use crate::core::{ASSETS, Abstract, join};
use crate::crossref::{Paper, domain};
use crate::jats;
use maud::{DOCTYPE, Escaper, Markup, PreEscaped, html};
use std::fmt::Write;

const COPY_JS: &str = "navigator.clipboard.writeText(document.querySelector('.bibtex').innerText)";

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

pub fn paper(paper: Paper, alternates: &[Paper], abstract_: Abstract) -> Markup {
    let title = paper.title();

    // Try converting the abstract from JATS XML to HTML we can render. If this
    // fails, just pass through the XML as text.
    // TODO we should probably log the error.
    let abs = match &abstract_ {
        Abstract::Jats(j) => {
            let content = match jats::to_html(j) {
                Ok(h) => html! { (PreEscaped(h)) },
                Err(_) => html! { (j) },
            };
            html! { div.abstract { (content) } }
        }
        Abstract::Text(t) => {
            html! { div.abstract {
                @for par in t.trim().split("\n\n") {
                    p { (par) }
                }
            } }
        }
        Abstract::Missing => {
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
                @if let Some(url) = paper.link_url() {
                    a href=(url) { ( domain(&url).unwrap() ) }
                }
                @if let Some(url) = paper.pdf_url() {
                    a href=(url) { "PDF" }
                }
            }
        }
        h1 { (title) };
        @if !paper.author.is_empty() {
            span.label { "Authors:" } " "
            div.authors {
                span.author { (paper.author[0].name()) }
                @for author in &paper.author[1..] {
                    ", "
                    span.author { (author.name()) }
                }
            }
        }
        span.label { "Published:" } " "
        div.published {
            @if paper.type_ == "journal-article" {
                ( paper.container_title.first().unwrap_or("Unknown journal") )
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
                (paper.event_title().unwrap_or("Unknown Event"))
                (". ")
                (paper.published)
                (".")
            }
        }
        @if !alternates.is_empty() {
            span.label { "Other Versions:" } " "
            ul.alternates {
                @for other_paper in alternates {
                    li {
                        a href=( format!("/doi/{}", other_paper.doi) ) { ( other_paper.doi ) }
                        ": "
                        ( other_paper.container_title.first().unwrap_or("Unknown") )
                    }
                }
            }
        }
        span.label { "Abstract:" } " "
        (abs)
        span.label {
            button.copy onclick=(COPY_JS) title="copy to clipboard" { (PreEscaped("&#x29C9;")) }
            "BibTeX:"
        } " "
        pre.bibtex {
            (bib::Entry(&paper))
        }
    };

    let doi_url = format!("https://doi.org/{}", paper.doi);
    let authors = join(paper.author.iter().map(|a| a.name()), ", ");
    let head = html! {
        meta property="og:title" content=(title);
        meta property="og:url" content=(doi_url);
        @if let Some(abs) = abstract_.text() {
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

    // Escape the JavaScript source for the bookmarklet.
    // TODO It would be nice to do this once at startup instead of on every render...
    let bm_src = asset("bookmarklet.js", host);
    let bm_src = bm_src.replace("\n", " ");
    let bm_src = bm_src.trim();
    let mut bm_escaped = String::new();
    Escaper::new(&mut bm_escaped).write_str(bm_src).unwrap();
    let home = home.replace("__BOOKMARKLET__", &bm_escaped);

    page(
        "Analog Library Premium Edition™",
        PreEscaped(home),
        html! {},
    )
}

/// Serve a static(ish) asset from our resources. Replace the __HOST__ token in
/// this asset with the server's host.
pub fn asset(filename: &str, host: &str) -> String {
    #[cfg(debug_assertions)]
    let script = ASSETS.read(filename).expect("asset must exist").unwrap();

    #[cfg(not(debug_assertions))]
    let script = ASSETS.get(filename).expect("asset must exist");

    script.replace("__HOST__", host)
}

pub fn doi_not_found(doi: &str) -> Markup {
    if doi.starts_with("10.5555/") {
        page(
            "404 Not Found: Fake DOI",
            html! {
                h1 { "404 Not Found: Fake DOI" }
                p {
                    ("This seems to be a fake DOI used in the ACM Digital Library. ")
                    ("The ACM DL appears to ")
                    a href="https://nickwalker.us/blog/2024/acm-dl-fake-dois/" {
                        ("use the ")
                        code { ("10.5555/") }
                        (" prefix for papers where it does not know the real DOI")
                    }
                    (", including many papers from non-ACM publishers. These fake DOIs ")
                    ("do not appear in the ")
                    a href="https://www.crossref.org" { "Crossref" }
                    (" database, so Analog Library cannot display them.")
                }
                p {
                    ("You can try viewing ")
                    a href=(format!("https://dl.acm.org/doi/{}", doi)) { "this DOI on the ACM DL" }
                    (" instead.")
                }
            },
            html! {},
        )
    } else {
        page(
            "404 Not Found",
            html! {
                h1 { "404 Not Found" }
                p {
                    ("Analog Library does not have data for the DOI ")
                    code { (doi) } (". ")
                    ("It uses the ")
                    a href="https://www.crossref.org" { "Crossref" }
                    (" database, so only DOIs present there can be rendered. ")
                    ("If this is a real DOI, try ")
                    a href=(format!("https://dx.doi.org/{}", doi)) { "the official DOI redirector" }
                    (" to get to the publisher page.")
                }
            },
            html! {},
        )
    }
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
