//! Markdown -> HTML for the live preview.
//!
//! Raw HTML from the note is turned into plain text before rendering, so an
//! imported `.md` file cannot inject markup or scripts into the page.

use pulldown_cmark::{html, Event, Options, Parser};

pub fn render(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options).map(|event| match event {
        Event::Html(raw) => Event::Text(raw),
        Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });

    let mut out = String::with_capacity(source.len() + 128);
    html::push_html(&mut out, parser);
    out
}
