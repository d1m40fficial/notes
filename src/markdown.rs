//! Markdown -> HTML for the live preview.
//!
//! Raw HTML from the note is turned into plain text before rendering, so an
//! imported `.md` file cannot inject markup or scripts into the page.

use pulldown_cmark::{html, Event, Options, Parser};

use std::collections::HashMap;

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
    activate_task_checkboxes(out)
}

/// Render a note whose images are stored under short ids like `img-n…`.
pub fn render_with_images(source: &str, images: &HashMap<String, String>) -> String {
    resolve_images(render(source), images)
}

fn resolve_images(html: String, images: &HashMap<String, String>) -> String {
    if !html.contains("src=\"img-") {
        return html;
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html.as_str();
    const PREFIX: &str = "src=\"";
    loop {
        let Some(pos) = rest.find(PREFIX) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..pos + PREFIX.len()]);
        rest = &rest[pos + PREFIX.len()..];
        let Some(end) = rest.find('"') else {
            out.push_str(rest);
            break;
        };
        let value = &rest[..end];
        match images.get(value) {
            Some(data) => out.push_str(data),
            None if value.starts_with("img-") => out.push_str(MISSING_IMAGE),
            None => out.push_str(value),
        }
        out.push('"');
        rest = &rest[end + 1..];
    }
    out
}

/// Shown instead of a broken image after the picture cache was cleared.
const MISSING_IMAGE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='480' height='160'%3E%3Crect width='480' height='160' rx='12' fill='%23efece6'/%3E%3Cpath d='M170 112l40-44 32 34 24-26 44 48H170z' fill='%23c9c2b6'/%3E%3Ccircle cx='300' cy='60' r='12' fill='%23c9c2b6'/%3E%3C/svg%3E";

/// pulldown-cmark renders task-list items as `disabled` checkboxes, which the
/// browser refuses to deliver clicks to. Re-enable them and number them so the
/// app can map a click back to the n-th `- [ ]` line of the note.
fn activate_task_checkboxes(html: String) -> String {
    const DISABLED: &str = "<input disabled=\"\" type=\"checkbox\"";
    if !html.contains(DISABLED) {
        return html;
    }
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html.as_str();
    let mut index = 0usize;
    while let Some(pos) = rest.find(DISABLED) {
        out.push_str(&rest[..pos]);
        out.push_str("<input type=\"checkbox\" data-task=\"");
        out.push_str(&index.to_string());
        out.push('"');
        rest = &rest[pos + DISABLED.len()..];
        index += 1;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn images_keep_their_data_url() {
        let html = super::render("![схема](data:image/png;base64,iVBORw0KGgo=)");
        assert!(html.contains("<img src=\"data:image/png;base64,iVBORw0KGgo=\""), "{html}");
        assert!(html.contains("alt=\"схема\""), "{html}");
    }

    #[test]
    fn stored_images_resolve_and_missing_ones_get_a_placeholder() {
        let mut images = HashMap::new();
        images.insert("img-test-1".to_string(), "data:image/png;base64,AAAA".to_string());
        let html = super::render_with_images(
            "![a](img-test-1)\n\n![b](img-test-2)",
            &images,
        );
        assert!(html.contains("data:image/png;base64,AAAA"), "{html}");
        assert!(html.contains("data:image/svg+xml"), "{html}");
        assert!(!html.contains("img-test-2"), "{html}");
    }

    #[test]
    fn task_checkboxes_are_interactive() {
        let html = super::render("- [ ] один\n- [x] два\n\n---\n");
        assert!(html.contains("data-task=\"0\""), "{html}");
        assert!(html.contains("data-task=\"1\""), "{html}");
        assert!(!html.contains("disabled"), "{html}");
        assert!(html.contains("checked"), "{html}");
        assert!(html.contains("<hr />"), "{html}");
    }
}
