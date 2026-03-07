use crate::ASSETS;
use crate::crossref::Paper;
use crate::jats;
use maud::{DOCTYPE, Markup, PreEscaped, html};

pub fn paper_page(paper: Paper, abstract_: Option<String>) -> Markup {
    #[cfg(not(debug_assertions))]
    let css = ASSETS.get("style.css").expect("asset must exist");

    #[cfg(debug_assertions)]
    let css = ASSETS.read("style.css").expect("asset must exist").unwrap();

    let title = paper.title();

    // Try converting the abstract from JATS XML to HTML we can render. If this
    // fails, just pass through the XML as text.
    // TODO we should probably log the error.
    let abs = match abstract_ {
        Some(j) => match jats::to_html(&j) {
            Ok(h) => html! { div.abstract { (PreEscaped(h)) } },
            Err(_) => html! { div.abstract { (j) } },
        },
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
                style { (PreEscaped(css)) };
            }
        }
        body {
            main {
                nav {
                    span.type {
                        ( paper.human_type() )
                    }
                    @if let Some(url) = paper.resource_url() {
                        a href=(url) { "paper" }
                    }
                    @if let Some(url) = paper.pdf_url() {
                        a href=(url) { "PDF" }
                    }
                }
                h1 { (title) };
                ul.authors {
                    @for author in &paper.author {
                        li { (author.name()) }
                    }
                };
                div.published {
                    @if paper.type_ == "journal-article" {
                        (paper.container_title)
                        (", volume ")
                        (paper.volume.as_deref().unwrap_or(""))
                        (", issue ")
                        (paper.issue.as_deref().unwrap_or(""))
                        (", pp. ")
                        (paper.page)
                        (".")
                    } @else if paper.type_ == "proceedings-article" {
                        ("In ")
                        (paper.event.as_deref().unwrap_or(""))
                        (", ")
                        (paper.published.year())
                        (".")
                    }
                }
                (abs)
            }
        }
    }
}
