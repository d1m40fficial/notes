//! Helpers for continuing and nesting Markdown lists inside a textarea.

#[derive(Clone, Copy, Debug)]
pub enum ListKind {
    Bullet(char),
    Ordered { number: u64, delimiter: char },
}

#[derive(Clone, Copy, Debug)]
pub struct ListLine {
    /// Byte offset where the leading whitespace ends.
    pub indent: usize,
    /// Byte offset where the bullet or number starts.
    pub marker_start: usize,
    /// Byte offset where the item text starts (after the marker and its space).
    pub content_start: usize,
    pub kind: ListKind,
}

impl ListLine {
    /// Width of `- `, `1. `, `10) ` and so on. Used as one nesting level.
    pub fn marker_width(&self) -> usize {
        self.content_start - self.marker_start
    }
}

/// Parse a Markdown list marker at the start of `line`.
///
/// Accepts `-`, `*`, `+` and ordered markers such as `1.` / `2)`.
/// The marker must be followed by whitespace or the end of the line, so
/// `---`, `*emphasis*` and `1.5` are not treated as list items.
pub fn parse_list_line(line: &str) -> Option<ListLine> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    let indent = i;
    let marker_start = i;
    if i >= bytes.len() {
        return None;
    }

    let kind = if matches!(bytes[i], b'-' | b'*' | b'+') {
        i += 1;
        ListKind::Bullet(bytes[i - 1] as char)
    } else if bytes[i].is_ascii_digit() {
        let number_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i >= bytes.len() || !matches!(bytes[i], b'.' | b')') {
            return None;
        }
        let number = line[number_start..i].parse::<u64>().ok()?;
        let delimiter = bytes[i] as char;
        i += 1;
        ListKind::Ordered { number, delimiter }
    } else {
        return None;
    };

    if i < bytes.len() && !matches!(bytes[i], b' ' | b'\t') {
        return None;
    }
    if i < bytes.len() {
        i += 1;
    }

    Some(ListLine {
        indent,
        marker_start,
        content_start: i,
        kind,
    })
}

/// Split an optional task-list checkbox off the content of a list item.
pub fn split_task(content: &str) -> (bool, &str) {
    let bytes = content.as_bytes();
    if bytes.len() >= 3
        && bytes[0] == b'['
        && bytes[2] == b']'
        && matches!(bytes[1], b' ' | b'x' | b'X')
        && (bytes.len() == 3 || matches!(bytes[3], b' ' | b'\t'))
    {
        let rest = if bytes.len() > 3 { &content[4..] } else { "" };
        return (true, rest);
    }
    (false, content)
}

/// Marker for the next item in the same list.
pub fn next_marker(kind: ListKind) -> String {
    match kind {
        ListKind::Bullet(ch) => format!("{ch} "),
        ListKind::Ordered { number, delimiter } => {
            format!("{}{delimiter} ", number.saturating_add(1))
        }
    }
}
