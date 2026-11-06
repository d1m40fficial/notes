use web_sys::Storage;

use crate::model::{normalize, Note, State, Theme, LEGACY_NOTE_KEY, STATE_KEY, THEME_KEY};

pub struct Loaded {
    pub state: State,
    pub theme: Theme,
    pub storage_ok: bool,
}

fn local_storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn load() -> Loaded {
    let storage = local_storage();
    let storage_ok = storage.is_some();

    let mut state = storage
        .as_ref()
        .and_then(|s| s.get_item(STATE_KEY).ok().flatten())
        .and_then(|raw| serde_json::from_str::<State>(&raw).ok())
        .unwrap_or_default();

    let theme = storage
        .as_ref()
        .and_then(|s| s.get_item(THEME_KEY).ok().flatten())
        .and_then(|raw| Theme::parse(&raw))
        .unwrap_or(Theme::Light);

    if let Some(s) = storage.as_ref() {
        if state.notes.is_empty() {
            if let Some(legacy) = s.get_item(LEGACY_NOTE_KEY).ok().flatten() {
                if !legacy.is_empty() {
                    state.notes.push(Note::from_body(legacy));
                }
                let _ = s.remove_item(LEGACY_NOTE_KEY);
            }
        }
    }

    normalize(&mut state);
    Loaded {
        state,
        theme,
        storage_ok,
    }
}

pub fn write_state(state: &State) -> Result<(), String> {
    let storage = local_storage().ok_or_else(|| "localStorage недоступен".to_string())?;
    let raw = serde_json::to_string(state).map_err(|err| err.to_string())?;
    storage
        .set_item(STATE_KEY, &raw)
        .map_err(|_| "не удалось записать localStorage".to_string())
}

pub fn write_theme(theme: Theme) -> Result<(), String> {
    let storage = local_storage().ok_or_else(|| "localStorage недоступен".to_string())?;
    storage
        .set_item(THEME_KEY, theme.as_str())
        .map_err(|_| "не удалось записать тему".to_string())
}
