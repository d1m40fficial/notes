use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::util::{now_ms, plural, time_ago};

pub const STATE_KEY: &str = "plain.state.v2";
pub const LEGACY_NOTE_KEY: &str = "plain.note.v1";
pub const THEME_KEY: &str = "plain.theme.v1";
pub const SAVE_DELAY_MS: i32 = 240;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub created_at: f64,
    #[serde(default)]
    pub updated_at: f64,
}

impl Note {
    pub fn empty() -> Self {
        Self::from_body("")
    }

    pub fn from_body(body: impl Into<String>) -> Self {
        let body = body.into();
        let now = now_ms();
        Self {
            id: crate::util::uid(),
            body,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn title(&self) -> String {
        let line = self
            .body
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("");
        let mut title = line.trim().trim_start_matches('#').trim().to_string();
        if title.is_empty() {
            return "Без названия".into();
        }
        let limit = 52;
        if title.chars().count() > limit {
            title = title.chars().take(limit).collect::<String>();
            title = title.trim_end().to_string();
            title.push('…');
        }
        title
    }

    pub fn word_count(&self) -> usize {
        self.body.split_whitespace().count()
    }

    pub fn char_count(&self) -> usize {
        self.body.chars().count()
    }

    pub fn reading_minutes(&self) -> usize {
        let words = self.word_count();
        if words == 0 {
            0
        } else {
            ((words + 199) / 200).max(1)
        }
    }

    pub fn words_label(&self) -> String {
        crate::util::words_label(self.word_count())
    }

    pub fn counts_label(&self) -> String {
        let words = self.word_count();
        let chars = self.char_count();
        format!(
            "{} {} · {} {} · {} мин",
            words,
            plural(words, "слово", "слова", "слов"),
            chars,
            plural(chars, "символ", "символа", "символов"),
            self.reading_minutes()
        )
    }

    pub fn list_meta(&self) -> String {
        if self.body.trim().is_empty() {
            format!("{} · пусто", time_ago(self.updated_at))
        } else {
            format!("{} · {}", time_ago(self.updated_at), self.words_label())
        }
    }

    pub fn pane_meta(&self) -> String {
        format!("обновлено {}", time_ago(self.updated_at))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Session {
    pub active: Option<String>,
    pub secondary: Option<String>,
    pub split: bool,
    pub sidebar: bool,
    pub active_pane: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            active: None,
            secondary: None,
            split: false,
            sidebar: true,
            active_pane: 0,
        }
    }
}

impl Session {
    pub fn pane_note(&self, pane: usize) -> Option<&str> {
        if pane == 1 {
            self.secondary.as_deref()
        } else {
            self.active.as_deref()
        }
    }

    pub fn is_open(&self, id: &str) -> bool {
        self.active.as_deref() == Some(id) || self.secondary.as_deref() == Some(id)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    #[serde(default)]
    pub notes: Vec<Note>,
    #[serde(default)]
    pub session: Session,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

/// Make a loaded or freshly mutated state internally consistent.
pub fn normalize(state: &mut State) {
    let now = now_ms();
    let mut seen = HashSet::new();
    for note in &mut state.notes {
        if note.id.is_empty() || !seen.insert(note.id.clone()) {
            note.id = crate::util::uid();
            seen.insert(note.id.clone());
        }
        if !note.created_at.is_finite() || note.created_at <= 0.0 {
            note.created_at = now;
        }
        if !note.updated_at.is_finite() || note.updated_at <= 0.0 {
            note.updated_at = note.created_at;
        }
    }

    if state.notes.is_empty() {
        state.notes.push(Note::empty());
    }

    let active_valid = state
        .session
        .active
        .as_deref()
        .map(|id| state.notes.iter().any(|note| note.id == id))
        .unwrap_or(false);
    if !active_valid {
        state.session.active = state.notes.first().map(|note| note.id.clone());
    }

    if !state.session.split {
        state.session.secondary = None;
        state.session.active_pane = 0;
        return;
    }

    let active = state.session.active.clone();
    let secondary_valid = state
        .session
        .secondary
        .as_deref()
        .map(|id| Some(id) != active.as_deref() && state.notes.iter().any(|note| note.id == id))
        .unwrap_or(false);
    if !secondary_valid {
        state.session.secondary = state
            .notes
            .iter()
            .find(|note| Some(note.id.as_str()) != active.as_deref())
            .map(|note| note.id.clone());
    }
    if state.session.secondary.is_none() {
        let note = Note::empty();
        let id = note.id.clone();
        state.notes.push(note);
        state.session.secondary = Some(id);
    }
    state.session.active_pane = state.session.active_pane.min(1);
}
