use wasm_bindgen::JsValue;

pub fn now_ms() -> f64 {
    js_sys::Date::now()
}

/// Short, collision-resistant id for a note.
pub fn uid() -> String {
    let t = now_ms() as u64;
    let r = (js_sys::Math::random() * 1_000_000_000.0) as u64;
    format!("n{t:x}-{r:x}")
}

pub fn plural<'a>(n: usize, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let m10 = n % 10;
    let m100 = n % 100;
    if m10 == 1 && m100 != 11 {
        one
    } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
        few
    } else {
        many
    }
}

pub fn words_label(n: usize) -> String {
    format!("{} {}", n, plural(n, "слово", "слова", "слов"))
}

pub fn time_ago(ts: f64) -> String {
    let diff = (now_ms() - ts).max(0.0);
    let minutes = (diff / 60_000.0).floor() as u64;
    if minutes < 1 {
        return "только что".into();
    }
    if minutes < 60 {
        return format!("{minutes} мин назад");
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{hours} ч назад");
    }
    let days = hours / 24;
    if days < 30 {
        return format!("{days} дн назад");
    }
    short_date(ts)
}

pub fn short_date(ts: f64) -> String {
    let d = js_sys::Date::new(&JsValue::from_f64(ts));
    format!(
        "{:04}-{:02}-{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date()
    )
}

pub fn date_stamp() -> String {
    short_date(now_ms())
}

/// Turn a note title into a safe, readable fragment for a downloaded file.
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '.') {
            if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
        }
        if out.chars().count() >= 40 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}

/// Convert a DOM UTF-16 selection offset into a UTF-8 byte offset.
/// If the offset lands inside a surrogate pair, the start of that char is used.
pub fn utf16_to_byte_index(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    let mut units = 0usize;
    for (byte, ch) in text.char_indices() {
        if units >= index {
            return byte;
        }
        if units + ch.len_utf16() > index {
            return byte;
        }
        units += ch.len_utf16();
    }
    text.len()
}

/// Convert a UTF-8 byte offset into a DOM UTF-16 code-unit offset.
pub fn byte_to_utf16_index(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    let mut units = 0usize;
    for (byte, ch) in text.char_indices() {
        if byte >= index {
            return units;
        }
        units += ch.len_utf16();
    }
    units
}
