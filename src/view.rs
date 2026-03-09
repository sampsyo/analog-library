use crate::ASSETS;
use crate::crossref::Paper;
use crate::jats;
use maud::{DOCTYPE, Markup, PreEscaped, html};

#[cfg(debug_assertions)]
fn css() -> String {
    ASSETS.read("style.css").expect("asset must exist").unwrap()
}

#[cfg(not(debug_assertions))]
fn css() -> &'static str {
    ASSETS.get("style.css").expect("asset must exist")
}

pub fn paper_page(paper: Paper, abstract_: Option<String>) -> Markup {
    let title = paper.title();

    // Try converting the abstract from JATS XML to HTML we can render. If this
    // fails, just pass through the XML as text.
    // TODO we should probably log the error.
    let abs = match abstract_ {
        Some(j) => {
            let content = match jats::to_html(&j) {
                Ok(h) => html! { (PreEscaped(h)) },
                Err(_) => html! { (j) },
            };
            html! {
                span.label { "Abstract:" } " "
                div.abstract { (content) }
            }
        }
        None => {
            html! { div.abstract.missing { "Abstract missing." } }
        }
    };

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (title) };
                style { (PreEscaped(css())) };
            }
        }
        body {
            main {
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
                        (", volume ")
                        (paper.volume.as_deref().unwrap_or(""))
                        (", issue ")
                        (paper.issue.as_deref().unwrap_or(""))
                        (", pp. ")
                        (paper.page)
                    } @else if paper.type_ == "proceedings-article" {
                        ("In ")
                        (paper.event.as_deref().unwrap_or(""))
                        (", ")
                        (paper.published.year())
                    }
                }
                (abs)
            }
        }
    }
}

pub fn home_page(host: &str) -> Markup {
    #[cfg(debug_assertions)]
    let home = ASSETS.read("home.html").expect("asset must exist").unwrap();

    #[cfg(not(debug_assertions))]
    let home = ASSETS.get("home.html").expect("asset must exist");

    let home = home.replace("__HOST__", host);
    let home = home.replace("__VERSION__", env!("CARGO_PKG_VERSION"));

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { ("Analog Library: Premium Edition") };
                style { (PreEscaped(css())) };
            }
        }
        body {
            main {
                (PreEscaped(home))
            }
        }
    }
}
