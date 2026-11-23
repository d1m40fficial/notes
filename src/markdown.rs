//! Markdown -> HTML for the live preview.
//!
//! Raw HTML from the note is turned into plain text before rendering, so an
//! imported `.md` file cannot inject markup or scripts into the page.
//!
//! The note is shown as a list of top-level blocks. Each block knows its byte
//! range in the source, so the UI can swap a single block for its raw text
//! while it is being edited.

use pulldown_cmark::{html, Event, Options, Parser};

fn options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn render(source: &str) -> String {
    render_from(source, 0)
}

/// Render markdown, numbering task checkboxes starting at `task_base`.
pub fn render_from(source: &str, task_base: usize) -> String {
    let parser = Parser::new_ext(source, options()).map(|event| match event {
        Event::Html(raw) => Event::Text(raw),
        Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });

    let mut out = String::with_capacity(source.len() + 128);
    html::push_html(&mut out, parser);
    activate_task_checkboxes(out, task_base)
}

/// Byte ranges of the top-level Markdown blocks of `source`.
pub fn blocks(source: &str) -> Vec<Block> {
    if source.trim().is_empty() {
        return Vec::new();
    }
    let mut depth = 0usize;
    let mut blocks: Vec<Block> = Vec::new();
    for (event, range) in Parser::new_ext(source, options()).into_offset_iter() {
        match event {
            Event::Start(_) => {
                if depth == 0 {
                    blocks.push(Block {
                        start: range.start,
                        end: range.end,
                    });
                }
                depth += 1;
            }
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(block) = blocks.last_mut() {
                        block.end = block.end.max(range.end);
                    }
                }
            }
            // Leaf blocks such as `---` have no Start/End pair.
            Event::Rule | Event::Html(_) if depth == 0 => {
                blocks.push(Block {
                    start: range.start,
                    end: range.end,
                });
            }
            _ => {}
        }
    }
    blocks.retain(|block| block.end > block.start);
    blocks
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    pub start: usize,
    pub end: usize,
}

/// Byte index of the `[` of a task-list marker, if the line is a task item.
pub fn task_marker(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    let mut digits = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        digits = true;
        i += 1;
    }
    if digits {
        if i >= bytes.len() || (bytes[i] != b'.' && bytes[i] != b')') {
            return None;
        }
        i += 1;
    } else if i < bytes.len() && matches!(bytes[i], b'-' | b'*' | b'+') {
        i += 1;
    } else {
        return None;
    }
    if i >= bytes.len() || bytes[i] != b' ' {
        return None;
    }
    i += 1;
    if i + 2 >= bytes.len() || bytes[i] != b'[' || bytes[i + 2] != b']' {
        return None;
    }
    if !matches!(bytes[i + 1], b' ' | b'x' | b'X') {
        return None;
    }
    Some(i)
}

/// pulldown-cmark renders task-list items as `disabled` checkboxes, which the
/// browser refuses to deliver clicks to. Re-enable them and number them so the
/// app can map a click back to the n-th `- [ ]` line of the note.
fn activate_task_checkboxes(html: String, task_base: usize) -> String {
    const DISABLED: &str = "<input disabled=\"\" type=\"checkbox\"";
    if !html.contains(DISABLED) {
        return html;
    }
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html.as_str();
    let mut index = task_base;
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
    use super::{blocks, render, render_from, task_marker};

    #[test]
    fn task_checkboxes_are_interactive() {
        let html = render("- [ ] один\n- [x] два\n\n---\n");
        assert!(html.contains("data-task=\"0\""), "{html}");
        assert!(html.contains("data-task=\"1\""), "{html}");
        assert!(!html.contains("disabled"), "{html}");
        assert!(html.contains("checked"), "{html}");
        assert!(html.contains("<hr />"), "{html}");
    }

    #[test]
    fn task_numbers_can_be_shifted() {
        let html = render_from("- [ ] пункт", 7);
        assert!(html.contains("data-task=\"7\""), "{html}");
    }

    #[test]
    fn blocks_cover_the_source() {
        let source = "# Итог\n\nПервый абзац.\n\n- [ ] раз\n- [x] два\n\n---\n\n```\nкод\n```\n";
        let found = blocks(source);
        let slices: Vec<&str> = found
            .iter()
            .map(|block| &source[block.start..block.end])
            .collect();
        assert_eq!(slices.len(), 5, "{slices:?}");
        assert!(slices[0].starts_with("# Итог"));
        assert!(slices[1].starts_with("Первый"));
        assert!(slices[2].starts_with("- [ ] раз"));
        assert!(slices[3].starts_with("---"));
        assert!(slices[4].starts_with("```"));
        for window in found.windows(2) {
            assert!(window[0].end <= window[1].start);
        }
    }

    #[test]
    fn empty_source_has_no_blocks() {
        assert!(blocks("").is_empty());
        assert!(blocks("   \n").is_empty());
    }

    #[test]
    fn task_marker_matches_task_lines_only() {
        assert!(task_marker("- [ ] a").is_some());
        assert!(task_marker("  * [x] b").is_some());
        assert!(task_marker("1. [ ] c").is_some());
        assert!(task_marker("- обычный пункт").is_none());
        assert!(task_marker("- [y] нет").is_none());
        assert!(task_marker("просто текст").is_none());
    }
}
