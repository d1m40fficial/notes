const PATHS: &[(&str, &str)] = &[
    ("plus", r#"<path d="M12 5v14M5 12h14"/>"#),
    (
        "panel",
        r#"<rect x="3.5" y="4.5" width="17" height="15" rx="2"/><path d="M9.5 4.5v15"/>"#,
    ),
    (
        "split",
        r#"<rect x="3.5" y="4.5" width="17" height="15" rx="2"/><path d="M12 4.5v15"/>"#,
    ),
    ("download", r#"<path d="M12 4v11m-4-4 4 4 4-4M5 19h14"/>"#),
    ("upload", r#"<path d="M12 20V9m-4 4 4-4 4 4M5 4h14"/>"#),
    (
        "rename",
        r#"<path d="M4 20h4L18.5 9.5a2.12 2.12 0 0 0-3-3L5 17v3Z"/><path d="m12.5 7.5 3 3"/>"#,
    ),
    (
        "trash",
        r#"<path d="M4.5 7h15M9 7V4.5h6V7M6.5 7l1 12.5h9l1-12.5M10 11v5M14 11v5"/>"#,
    ),
    (
        "sun",
        r#"<circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.2 5.2l1.4 1.4M17.4 17.4l1.4 1.4M18.8 5.2l-1.4 1.4M6.6 17.4l-1.4 1.4"/>"#,
    ),
    (
        "moon",
        r#"<path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z"/>"#,
    ),
    (
        "notes",
        r#"<path d="M6 3.5h9.5L19 7v13.5H6z"/><path d="M15 3.5V7h4M9 11h7M9 14.5h7M9 18h4"/>"#,
    ),
    ("check", r#"<path d="m5 12.5 4.5 4.5L19 7"/>"#),
    (
        "warning",
        r#"<path d="M12 4 21 20H3L12 4Z"/><path d="M12 10v4.5M12 17.5h.01"/>"#,
    ),
];

pub fn icon(name: &str, class: &str) -> String {
    let body = PATHS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, path)| *path)
        .unwrap_or(PATHS[0].1);
    format!(
        r#"<svg class="icon {class}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{body}</svg>"#
    )
}
