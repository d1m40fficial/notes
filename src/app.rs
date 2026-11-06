use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, BlobPropertyBag, Document, Element, Event, HtmlAnchorElement, HtmlElement,
    HtmlTextAreaElement, KeyboardEvent, Window,
};

use crate::icons::icon;
use crate::model::{normalize, Note, Session, State, Theme, SAVE_DELAY_MS};
use crate::storage;
use crate::util::{self, plural};

const MOBILE_BREAKPOINT: f64 = 780.0;

pub fn mount() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window недоступен"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document недоступен"))?;
    let root = document
        .get_element_by_id("app")
        .ok_or_else(|| JsValue::from_str("не найден #app"))?;

    let loaded = storage::load();
    set_theme(&document, loaded.theme);
    root.set_inner_html(&skeleton());

    let app = App::new(window, document, root, loaded.state, loaded.theme);
    app.render_all();
    app.wire()?;

    if loaded.storage_ok {
        let write = {
            let state = app.state.borrow();
            storage::write_state(&state)
        };
        if write.is_err() {
            app.set_status("ошибка", "is-error");
        }
    } else {
        app.set_status("без сохранения", "is-error");
    }

    app.focus_if_wide();
    Ok(())
}

fn set_theme(document: &Document, theme: Theme) {
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("data-theme", theme.as_str());
    }
    if let Ok(Some(meta)) = document.query_selector("meta[name=\"theme-color\"]") {
        let _ = meta.set_attribute(
            "content",
            match theme {
                Theme::Light => "#fbfaf7",
                Theme::Dark => "#0c0c0c",
            },
        );
    }
}

fn skeleton() -> String {
    format!(
        r##"<div class="app" id="shell" data-sidebar="open" data-split="off">
  <aside class="rail" aria-label="Быстрые действия">
    <span class="rail__brand" aria-hidden="true">p<span>.</span></span>
    <button class="rail__btn rail__btn--new" type="button" data-act="new" title="Новая заметка (Ctrl+Alt+N)" aria-label="Новая заметка">{plus}</button>
    <button class="rail__btn" id="sidebar-toggle" type="button" data-act="sidebar" title="Скрыть список (Ctrl+\)" aria-label="Скрыть список заметок" aria-pressed="true">{panel}</button>
    <span class="rail__spacer"></span>
    <button class="rail__btn" id="theme-toggle" type="button" data-act="theme" title="Тёмная тема" aria-label="Сменить тему">{moon}</button>
  </aside>

  <aside class="sidebar" id="sidebar" aria-label="Список заметок">
    <div class="sidebar__inner">
      <div class="sidebar__head">
        <span class="sidebar__title">заметки</span>
        <span class="sidebar__count" id="note-count">0</span>
      </div>
      <div class="sidebar__list" id="note-list" role="list"></div>
      <div class="sidebar__foot">
        <span class="sidebar__local">локально</span>
        <span id="sidebar-meta">0 заметок</span>
      </div>
    </div>
  </aside>

  <div class="scrim" data-act="sidebar-close" aria-hidden="true"></div>

  <main class="workspace">
    <header class="topbar">
      <span class="wordmark">plain<span>.</span></span>
      <span class="save-state" id="save-state" aria-live="polite">локально</span>
      <nav class="topbar__actions" aria-label="Действия">
        <button class="action" id="split-toggle" type="button" data-act="split" aria-pressed="false" title="Разделить экран (Ctrl+Alt+2)">
          {split}<span class="action__label">две</span>
        </button>
        <button class="action" type="button" data-act="export" title="Скачать .md (Ctrl+S)">
          {download}<span class="action__label">export</span>
        </button>
        <button class="action" type="button" data-act="clear" title="Очистить заметку">
          {trash}<span class="action__label">clear</span>
        </button>
      </nav>
    </header>

    <div class="panes" id="panes" data-split="off">
      <section class="pane pane--first is-active" data-pane="0" aria-label="Заметка 1">
        <header class="pane__head">
          <span class="pane__index" aria-hidden="true">1</span>
          <span class="pane__title" id="pane-title-0">Без названия</span>
          <span class="pane__meta" id="pane-meta-0"></span>
        </header>
        <div class="pane__paper">
          <label class="sr-only" for="editor-0">Заметка 1</label>
          <textarea class="editor" id="editor-0" data-editor="0" spellcheck="true" autocomplete="off" placeholder="Начните печатать. Всё сохраняется в этом браузере."></textarea>
        </div>
        <footer class="pane__foot">
          <span id="counts-0">0 слов · 0 символов · 0 мин</span>
          <span class="pane__hint">autosave · ctrl+s export</span>
        </footer>
      </section>

      <section class="pane pane--second" data-pane="1" aria-label="Заметка 2" aria-hidden="true">
        <header class="pane__head">
          <span class="pane__index" aria-hidden="true">2</span>
          <span class="pane__title" id="pane-title-1">Вторая заметка</span>
          <span class="pane__meta" id="pane-meta-1"></span>
          <button class="pane__close" id="pane-close" type="button" data-act="pane-close" title="Закрыть вторую заметку" aria-label="Закрыть вторую заметку" hidden>{close}</button>
        </header>
        <div class="pane__paper">
          <label class="sr-only" for="editor-1">Заметка 2</label>
          <textarea class="editor" id="editor-1" data-editor="1" spellcheck="true" autocomplete="off" placeholder="Вторая заметка. Она сохраняется отдельно."></textarea>
        </div>
        <footer class="pane__foot">
          <span id="counts-1">0 слов · 0 символов · 0 мин</span>
          <span class="pane__hint">вторая половина</span>
        </footer>
      </section>
    </div>
  </main>
</div>"##,
        plus = icon("plus", "icon--lg"),
        panel = icon("panel", ""),
        moon = icon("moon", ""),
        split = icon("split", ""),
        download = icon("download", ""),
        trash = icon("trash", ""),
        close = icon("close", ""),
    )
}

struct App {
    window: Window,
    document: Document,
    root: Element,
    shell: Element,
    sidebar: Element,
    list: Element,
    note_count: Element,
    sidebar_meta: Element,
    save_state: Element,
    split_button: Element,
    sidebar_button: Element,
    theme_button: Element,
    pane_close: Element,
    panes_container: Element,
    panes: Vec<Element>,
    editors: Vec<HtmlTextAreaElement>,
    pane_titles: Vec<Element>,
    pane_metas: Vec<Element>,
    pane_counts: Vec<Element>,
    state: RefCell<State>,
    theme: Cell<Theme>,
    save_timer: Cell<Option<i32>>,
    status_timer: Cell<Option<i32>>,
    self_ref: RefCell<Weak<App>>,
    closures: RefCell<Vec<Closure<dyn FnMut(Event)>>>,
}

impl App {
    fn new(
        window: Window,
        document: Document,
        root: Element,
        state: State,
        theme: Theme,
    ) -> Rc<Self> {
        let shell = query_el(&root, ".app");
        let sidebar = query_el(&root, "#sidebar");
        let list = query_el(&root, "#note-list");
        let note_count = query_el(&root, "#note-count");
        let sidebar_meta = query_el(&root, "#sidebar-meta");
        let save_state = query_el(&root, "#save-state");
        let split_button = query_el(&root, "#split-toggle");
        let sidebar_button = query_el(&root, "#sidebar-toggle");
        let theme_button = query_el(&root, "#theme-toggle");
        let pane_close = query_el(&root, "#pane-close");
        let panes_container = query_el(&root, "#panes");

        let panes = vec![
            query_el(&root, ".pane[data-pane=\"0\"]"),
            query_el(&root, ".pane[data-pane=\"1\"]"),
        ];
        let editors = vec![
            query_typed::<HtmlTextAreaElement>(&root, "#editor-0"),
            query_typed::<HtmlTextAreaElement>(&root, "#editor-1"),
        ];
        let pane_titles = vec![
            query_el(&root, "#pane-title-0"),
            query_el(&root, "#pane-title-1"),
        ];
        let pane_metas = vec![
            query_el(&root, "#pane-meta-0"),
            query_el(&root, "#pane-meta-1"),
        ];
        let pane_counts = vec![query_el(&root, "#counts-0"), query_el(&root, "#counts-1")];

        let app = Rc::new(Self {
            window,
            document,
            root,
            shell,
            sidebar,
            list,
            note_count,
            sidebar_meta,
            save_state,
            split_button,
            sidebar_button,
            theme_button,
            pane_close,
            panes_container,
            panes,
            editors,
            pane_titles,
            pane_metas,
            pane_counts,
            state: RefCell::new(state),
            theme: Cell::new(theme),
            save_timer: Cell::new(None),
            status_timer: Cell::new(None),
            self_ref: RefCell::new(Weak::new()),
            closures: RefCell::new(Vec::new()),
        });
        *app.self_ref.borrow_mut() = Rc::downgrade(&app);
        app
    }

    fn wire(&self) -> Result<(), JsValue> {
        let this = self
            .self_ref
            .borrow()
            .upgrade()
            .ok_or_else(|| JsValue::from_str("app"))?;

        let click_this = Rc::clone(&this);
        let click = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            click_this.on_click(event);
        });
        self.root
            .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
        self.closures.borrow_mut().push(click);

        let input_this = Rc::clone(&this);
        let input = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            input_this.on_input(event);
        });
        self.root
            .add_event_listener_with_callback("input", input.as_ref().unchecked_ref())?;
        self.closures.borrow_mut().push(input);

        let focus_this = Rc::clone(&this);
        let focus = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            focus_this.on_focus_in(event);
        });
        self.root
            .add_event_listener_with_callback("focusin", focus.as_ref().unchecked_ref())?;
        self.closures.borrow_mut().push(focus);

        let key_this = Rc::clone(&this);
        let key = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            key_this.on_key_down(event);
        });
        self.document
            .add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref())?;
        self.closures.borrow_mut().push(key);

        let unload_this = Rc::clone(&this);
        let unload = Closure::<dyn FnMut(Event)>::new(move |_: Event| {
            unload_this.persist_quiet();
        });
        self.window
            .add_event_listener_with_callback("beforeunload", unload.as_ref().unchecked_ref())?;
        self.closures.borrow_mut().push(unload);

        Ok(())
    }

    fn now(&self) -> f64 {
        util::now_ms()
    }

    fn on_click(&self, event: Event) {
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
        else {
            return;
        };

        if let Some(pane_el) = target.closest(".pane").ok().flatten() {
            if let Some(pane) = pane_el
                .get_attribute("data-pane")
                .and_then(|value| value.parse::<usize>().ok())
            {
                self.activate_pane(pane);
            }
        }

        let Some(action_el) = target.closest("[data-act]").ok().flatten() else {
            return;
        };
        let action = action_el.get_attribute("data-act").unwrap_or_default();

        match action.as_str() {
            "new" => self.create_note(),
            "sidebar" => self.toggle_sidebar(),
            "sidebar-close" => self.set_sidebar(false, true),
            "theme" => self.toggle_theme(),
            "split" => self.toggle_split(),
            "export" => self.export_active(),
            "clear" => self.clear_active(),
            "pane-close" => self.close_split(),
            "note-open" | "note-split" | "note-delete" => {
                let Some(item) = target.closest(".note-item").ok().flatten() else {
                    return;
                };
                let Some(id) = item.get_attribute("data-id") else {
                    return;
                };
                match action.as_str() {
                    "note-open" => self.open_note(&id, None, true),
                    "note-split" => self.split_note(&id),
                    "note-delete" => self.delete_note(&id),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn on_input(&self, event: Event) {
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
        else {
            return;
        };
        let Some(editor_el) = target.closest("[data-editor]").ok().flatten() else {
            return;
        };
        let Some(pane) = editor_el
            .get_attribute("data-editor")
            .and_then(|value| value.parse::<usize>().ok())
        else {
            return;
        };
        self.handle_input(pane);
    }

    fn on_focus_in(&self, event: Event) {
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
        else {
            return;
        };
        let Some(editor_el) = target.closest("[data-editor]").ok().flatten() else {
            return;
        };
        let Some(pane) = editor_el
            .get_attribute("data-editor")
            .and_then(|value| value.parse::<usize>().ok())
        else {
            return;
        };
        self.activate_pane(pane);
    }

    fn on_key_down(&self, event: Event) {
        let target_editor = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .and_then(|el| el.closest("[data-editor]").ok().flatten())
            .and_then(|el| el.dyn_into::<HtmlTextAreaElement>().ok());

        let Ok(key_event) = event.dyn_into::<KeyboardEvent>() else {
            return;
        };

        let modifier = key_event.ctrl_key() || key_event.meta_key();
        let code = key_event.code();
        let key = key_event.key();

        if modifier && code == "KeyS" && !key_event.repeat() {
            key_event.prevent_default();
            self.export_active();
            return;
        }

        if modifier && key_event.alt_key() && code == "KeyN" && !key_event.repeat() {
            key_event.prevent_default();
            self.create_note();
            return;
        }

        if modifier
            && key_event.alt_key()
            && (code == "Digit2" || key == "2" || key == "@")
            && !key_event.repeat()
        {
            key_event.prevent_default();
            self.toggle_split();
            return;
        }

        if modifier && (code == "Backslash" || key == "\\" || key == "|") && !key_event.repeat() {
            key_event.prevent_default();
            self.toggle_sidebar();
            return;
        }

        if key == "Escape" {
            if self.is_narrow() {
                let open = { self.state.borrow().session.sidebar };
                if open {
                    self.set_sidebar(false, true);
                }
            }
            return;
        }

        if key == "Tab" && !key_event.shift_key() {
            if let Some(editor) = target_editor {
                key_event.prevent_default();
                let pane = editor
                    .get_attribute("data-editor")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);
                self.insert_two_spaces(&editor, pane);
            }
        }
    }

    fn insert_two_spaces(&self, editor: &HtmlTextAreaElement, pane: usize) {
        let value = editor.value();
        let start = editor.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = editor
            .selection_end()
            .ok()
            .flatten()
            .unwrap_or(start as u32) as usize;
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let start_byte = util::utf16_to_byte_index(&value, start);
        let end_byte = util::utf16_to_byte_index(&value, end);
        let mut next = String::with_capacity(value.len() + 2);
        next.push_str(&value[..start_byte]);
        next.push_str("  ");
        next.push_str(&value[end_byte..]);
        editor.set_value(&next);
        let caret = (start + 2) as u32;
        let _ = editor.set_selection_range(caret, caret);
        self.handle_input(pane);
    }

    fn handle_input(&self, pane: usize) {
        let Some(editor) = self.editors.get(pane) else {
            return;
        };
        let body = editor.value();
        let now = self.now();

        let info = {
            let mut state = self.state.borrow_mut();
            let id = match state.session.pane_note(pane) {
                Some(id) => id.to_string(),
                None => return,
            };
            let Some(note) = state.notes.iter_mut().find(|note| note.id == id) else {
                return;
            };
            if note.body == body {
                return;
            }
            note.body = body;
            note.updated_at = now;
            (
                note.id.clone(),
                note.title(),
                note.pane_meta(),
                note.counts_label(),
                note.list_meta(),
            )
        };

        let (id, title, pane_meta, counts, list_meta) = info;
        if let Some(node) = self.pane_titles.get(pane) {
            node.set_text_content(Some(&title));
            let _ = node.set_attribute("title", &title);
        }
        if let Some(node) = self.pane_metas.get(pane) {
            node.set_text_content(Some(&pane_meta));
        }
        if let Some(node) = self.pane_counts.get(pane) {
            node.set_text_content(Some(&counts));
        }
        self.update_list_item(&id, &title, &list_meta);
        self.update_document_title();
        self.set_status("сохраняю…", "is-saving");
        self.schedule_save();
    }

    fn schedule_save(&self) {
        self.clear_timeout(&self.save_timer);
        let weak = self.self_ref.borrow().clone();
        let Some(this) = weak.upgrade() else {
            return;
        };
        let callback = Closure::once_into_js(move || {
            this.save_now();
        });
        match self
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.unchecked_ref::<js_sys::Function>(),
                SAVE_DELAY_MS,
            ) {
            Ok(handle) => self.save_timer.set(Some(handle)),
            Err(_) => self.set_status("ошибка", "is-error"),
        }
    }

    fn save_now(&self) {
        self.clear_timeout(&self.save_timer);
        let result = {
            let state = self.state.borrow();
            storage::write_state(&state)
        };
        match result {
            Ok(()) => self.flash("сохранено", "is-saved", 1600),
            Err(_) => self.set_status("ошибка", "is-error"),
        }
    }

    fn save_quiet(&self) {
        let result = {
            let state = self.state.borrow();
            storage::write_state(&state)
        };
        if result.is_err() && self.save_state.text_content().as_deref() != Some("без сохранения")
        {
            self.set_status("ошибка", "is-error");
        }
    }

    fn persist_quiet(&self) {
        let _ = {
            let state = self.state.borrow();
            storage::write_state(&state)
        };
    }

    fn flash(&self, text: &str, class: &str, ms: i32) {
        self.set_status(text, class);
        self.clear_timeout(&self.status_timer);
        let weak = self.self_ref.borrow().clone();
        let Some(this) = weak.upgrade() else {
            return;
        };
        let callback = Closure::once_into_js(move || {
            this.set_status("локально", "");
        });
        match self
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.unchecked_ref::<js_sys::Function>(),
                ms,
            ) {
            Ok(handle) => self.status_timer.set(Some(handle)),
            Err(_) => {}
        }
    }

    fn set_status(&self, text: &str, class: &str) {
        self.clear_timeout(&self.status_timer);
        self.save_state.set_text_content(Some(text));
        for name in ["is-saving", "is-saved", "is-error"] {
            set_class(&self.save_state, name, false);
        }
        if !class.is_empty() {
            set_class(&self.save_state, class, true);
        }
    }

    fn clear_timeout(&self, cell: &Cell<Option<i32>>) {
        if let Some(handle) = cell.take() {
            self.window.clear_timeout_with_handle(handle);
        }
    }

    fn create_note(&self) {
        {
            let mut state = self.state.borrow_mut();
            let note = Note::empty();
            let id = note.id.clone();
            state.notes.insert(0, note);
            let pane = if state.session.split {
                state.session.active_pane.min(1)
            } else {
                0
            };
            if pane == 1 {
                state.session.secondary = Some(id);
            } else {
                state.session.active = Some(id);
            }
            normalize(&mut state);
        }
        self.save_now();
        self.render_all();
        let _ = self.list.set_scroll_top(0);
        if self.is_narrow() {
            self.set_sidebar(false, false);
        }
        self.focus_if_wide();
    }

    fn open_note(&self, id: &str, pane: Option<usize>, focus: bool) {
        let target_pane = {
            let state = self.state.borrow();
            match pane {
                Some(pane) => pane.min(1),
                None => {
                    if state.session.split {
                        state.session.active_pane.min(1)
                    } else {
                        0
                    }
                }
            }
        };
        let exists = {
            let state = self.state.borrow();
            state.notes.iter().any(|note| note.id == id)
        };
        if !exists {
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            if !state.session.split {
                state.session.active = Some(id.to_string());
                state.session.active_pane = 0;
            } else if target_pane == 0 {
                if state.session.active.as_deref() != Some(id) {
                    if state.session.secondary.as_deref() == Some(id) {
                        let previous = state.session.active.clone();
                        state.session.active = Some(id.to_string());
                        state.session.secondary = previous;
                    } else {
                        state.session.active = Some(id.to_string());
                    }
                }
                state.session.active_pane = 0;
            } else {
                if state.session.secondary.as_deref() != Some(id) {
                    if state.session.active.as_deref() == Some(id) {
                        let previous = state.session.secondary.clone();
                        state.session.secondary = Some(id.to_string());
                        state.session.active = previous;
                    } else {
                        state.session.secondary = Some(id.to_string());
                    }
                }
                state.session.active_pane = 1;
            }
            normalize(&mut state);
        }

        self.save_quiet();
        if self.is_narrow() {
            self.set_sidebar(false, false);
        }
        self.render_all();
        if focus {
            self.focus_if_wide();
        }
    }

    fn split_note(&self, id: &str) {
        {
            let mut state = self.state.borrow_mut();
            if !state.notes.iter().any(|note| note.id == id) {
                return;
            }
            state.session.split = true;
            if state.session.active.as_deref() == Some(id) {
                let replacement = state
                    .session
                    .secondary
                    .clone()
                    .filter(|candidate| candidate != id)
                    .or_else(|| {
                        state
                            .notes
                            .iter()
                            .find(|note| note.id != id)
                            .map(|note| note.id.clone())
                    });
                match replacement {
                    Some(replacement) => state.session.active = Some(replacement),
                    None => {
                        let note = Note::empty();
                        let new_id = note.id.clone();
                        state.notes.push(note);
                        state.session.active = Some(new_id);
                    }
                }
                state.session.secondary = Some(id.to_string());
            } else {
                state.session.secondary = Some(id.to_string());
            }
            state.session.active_pane = 1;
            normalize(&mut state);
        }

        self.save_quiet();
        if self.is_narrow() {
            self.set_sidebar(false, false);
        }
        self.render_all();
        self.focus_if_wide();
    }

    fn delete_note(&self, id: &str) {
        let title = {
            let state = self.state.borrow();
            match state.notes.iter().find(|note| note.id == id) {
                Some(note) => note.title(),
                None => return,
            }
        };
        let message = format!("Удалить заметку «{title}»? Это действие нельзя отменить.");
        if !self.window.confirm_with_message(&message).unwrap_or(false) {
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            let was_active = state.session.active.as_deref() == Some(id);
            let was_secondary = state.session.secondary.as_deref() == Some(id);
            state.notes.retain(|note| note.id != id);

            if was_active {
                if state.session.split {
                    state.session.active = state.session.secondary.take();
                    state.session.split = false;
                } else {
                    state.session.active = None;
                }
                state.session.active_pane = 0;
            } else if was_secondary {
                state.session.secondary = None;
                state.session.split = false;
                state.session.active_pane = 0;
            }
            normalize(&mut state);
        }

        self.save_quiet();
        self.render_all();
        self.focus_if_wide();
        self.flash("удалено", "is-saved", 1400);
    }

    fn clear_active(&self) {
        let is_empty = {
            let state = self.state.borrow();
            let pane = state.session.active_pane.min(1);
            state
                .session
                .pane_note(pane)
                .and_then(|id| state.notes.iter().find(|note| note.id == id))
                .map(|note| note.body.is_empty())
                .unwrap_or(true)
        };
        if is_empty {
            return;
        }
        if !self
            .window
            .confirm_with_message("Очистить заметку? Отменить это будет нельзя.")
            .unwrap_or(false)
        {
            return;
        }

        {
            let mut state = self.state.borrow_mut();
            let pane = state.session.active_pane.min(1);
            let id = state.session.pane_note(pane).map(str::to_string);
            if let Some(id) = id {
                let now = self.now();
                if let Some(note) = state.notes.iter_mut().find(|note| note.id == id) {
                    note.body.clear();
                    note.updated_at = now;
                }
            }
        }

        self.save_now();
        self.render_all();
        self.focus_if_wide();
        self.flash("очищено", "is-saved", 1400);
    }

    fn export_active(&self) {
        let (body, title) = {
            let state = self.state.borrow();
            let pane = state.session.active_pane.min(1);
            match state
                .session
                .pane_note(pane)
                .and_then(|id| state.notes.iter().find(|note| note.id == id))
            {
                Some(note) => (note.body.clone(), note.title()),
                None => return,
            }
        };

        self.save_now();
        let filename = export_filename(&title);
        match self.download(&body, &filename) {
            Ok(()) => self.flash("экспорт", "is-saved", 1400),
            Err(_) => self.set_status("ошибка", "is-error"),
        }
    }

    fn download(&self, body: &str, filename: &str) -> Result<(), JsValue> {
        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(body));
        let options = BlobPropertyBag::new();
        options.set_type("text/markdown;charset=utf-8");
        let blob = Blob::new_with_str_sequence_and_options(&parts, &options)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let anchor = self
            .document
            .create_element("a")?
            .dyn_into::<HtmlAnchorElement>()
            .map_err(|_| JsValue::from_str("не удалось создать ссылку"))?;
        anchor.set_href(&url);
        anchor.set_download(filename);
        let body_el = self
            .document
            .body()
            .ok_or_else(|| JsValue::from_str("нет body"))?;
        body_el.append_child(&anchor)?;
        anchor.click();
        anchor.remove();

        let callback = Closure::once_into_js(move || {
            let _ = web_sys::Url::revoke_object_url(&url);
        });
        self.window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.unchecked_ref::<js_sys::Function>(),
                1000,
            )?;
        Ok(())
    }

    fn toggle_split(&self) {
        let is_on = { self.state.borrow().session.split };
        if is_on {
            self.close_split();
            return;
        }
        {
            let mut state = self.state.borrow_mut();
            state.session.split = true;
            state.session.active_pane = 1;
            normalize(&mut state);
        }
        self.save_quiet();
        self.render_all();
        self.focus_if_wide();
    }

    /// Collapse the split back to the primary (left) note.
    fn close_split(&self) {
        {
            let mut state = self.state.borrow_mut();
            if !state.session.split {
                return;
            }
            state.session.secondary = None;
            state.session.split = false;
            state.session.active_pane = 0;
            normalize(&mut state);
        }
        self.save_quiet();
        self.render_all();
        self.focus_if_wide();
    }

    fn toggle_sidebar(&self) {
        let open = {
            let mut state = self.state.borrow_mut();
            state.session.sidebar = !state.session.sidebar;
            state.session.sidebar
        };
        self.apply_sidebar(open);
        self.save_quiet();
        if !open {
            self.focus_if_wide();
        }
    }

    fn set_sidebar(&self, open: bool, focus_editor: bool) {
        let changed = {
            let mut state = self.state.borrow_mut();
            if state.session.sidebar == open {
                false
            } else {
                state.session.sidebar = open;
                true
            }
        };
        if !changed {
            return;
        }
        self.apply_sidebar(open);
        self.save_quiet();
        if !open && focus_editor {
            self.focus_if_wide();
        }
    }

    fn apply_sidebar(&self, open: bool) {
        if !open {
            self.blur_sidebar_focus();
        }
        let _ = self
            .shell
            .set_attribute("data-sidebar", if open { "open" } else { "closed" });
        let _ = self
            .sidebar
            .set_attribute("aria-hidden", if open { "false" } else { "true" });
        if open {
            let _ = self.sidebar.remove_attribute("inert");
        } else {
            let _ = self.sidebar.set_attribute("inert", "");
        }
        let _ = self
            .sidebar_button
            .set_attribute("aria-pressed", if open { "true" } else { "false" });
        let label = if open {
            "Скрыть список заметок"
        } else {
            "Показать список заметок"
        };
        let _ = self.sidebar_button.set_attribute("aria-label", label);
        let title = if open {
            "Скрыть список (Ctrl+\\)"
        } else {
            "Показать список (Ctrl+\\)"
        };
        let _ = self.sidebar_button.set_attribute("title", title);
    }

    /// Do not leave keyboard focus inside an inert, translated-away sidebar.
    fn blur_sidebar_focus(&self) {
        let Some(active) = self.document.active_element() else {
            return;
        };
        if active.closest("#sidebar").ok().flatten().is_none() {
            return;
        }
        if let Ok(element) = active.dyn_into::<HtmlElement>() {
            let _ = element.blur();
        }
    }

    fn toggle_theme(&self) {
        let next = self.theme.get().next();
        self.theme.set(next);
        set_theme(&self.document, next);
        let _ = storage::write_theme(next);
        self.update_theme_button();
    }

    fn activate_pane(&self, pane: usize) {
        if pane > 1 {
            return;
        }
        let changed = {
            let mut state = self.state.borrow_mut();
            if !state.session.split && pane == 1 {
                return;
            }
            if state.session.active_pane == pane {
                false
            } else {
                state.session.active_pane = pane;
                true
            }
        };
        if changed {
            self.save_quiet();
            self.render_active_states();
        }
    }

    fn render_active_states(&self) {
        let (split, active_pane) = {
            let state = self.state.borrow();
            (state.session.split, state.session.active_pane.min(1))
        };
        for pane in 0..2 {
            let active = if split {
                pane == active_pane
            } else {
                pane == 0
            };
            set_class(&self.panes[pane], "is-active", active);
        }
        self.render_list();
        self.update_document_title();
    }

    fn render_all(&self) {
        let (sidebar, split) = {
            let state = self.state.borrow();
            (state.session.sidebar, state.session.split)
        };
        let _ = self
            .shell
            .set_attribute("data-sidebar", if sidebar { "open" } else { "closed" });
        let _ = self
            .shell
            .set_attribute("data-split", if split { "on" } else { "off" });
        let _ = self
            .panes_container
            .set_attribute("data-split", if split { "on" } else { "off" });
        self.apply_sidebar(sidebar);

        let _ = self
            .split_button
            .set_attribute("aria-pressed", if split { "true" } else { "false" });
        set_class(&self.split_button, "is-on", split);
        if let Ok(Some(label)) = self.split_button.query_selector(".action__label") {
            label.set_text_content(Some(if split { "одна" } else { "две" }));
        }
        let split_title = if split {
            "Закрыть вторую заметку (Ctrl+Alt+2)"
        } else {
            "Разделить экран (Ctrl+Alt+2)"
        };
        let _ = self.split_button.set_attribute("title", split_title);
        let _ = self.split_button.set_attribute(
            "aria-label",
            if split {
                "Закрыть вторую заметку"
            } else {
                "Разделить экран"
            },
        );

        self.update_theme_button();
        self.render_panes();
        self.render_list();
        self.update_document_title();
    }

    fn render_panes(&self) {
        let state = self.state.borrow();
        let split = state.session.split;
        let active_pane = state.session.active_pane.min(1);

        for pane in 0..2 {
            let note_id = state.session.pane_note(pane).map(str::to_string);
            if let Some(id) = note_id.as_deref() {
                let _ = self.panes[pane].set_attribute("data-note-id", id);
            } else {
                let _ = self.panes[pane].remove_attribute("data-note-id");
            }
            let note = note_id
                .as_deref()
                .and_then(|id| state.notes.iter().find(|note| note.id == id));
            if let Some(note) = note {
                let title = note.title();
                if let Some(node) = self.pane_titles.get(pane) {
                    node.set_text_content(Some(&title));
                    let _ = node.set_attribute("title", &title);
                }
                if let Some(node) = self.pane_metas.get(pane) {
                    node.set_text_content(Some(&note.pane_meta()));
                }
                if let Some(node) = self.pane_counts.get(pane) {
                    node.set_text_content(Some(&note.counts_label()));
                }
                if let Some(editor) = self.editors.get(pane) {
                    if editor.value() != note.body {
                        editor.set_value(&note.body);
                    }
                }
            } else {
                if let Some(node) = self.pane_titles.get(pane) {
                    node.set_text_content(Some("—"));
                }
                if let Some(node) = self.pane_metas.get(pane) {
                    node.set_text_content(Some(""));
                }
                if let Some(node) = self.pane_counts.get(pane) {
                    node.set_text_content(Some(""));
                }
                if let Some(editor) = self.editors.get(pane) {
                    editor.set_value("");
                }
            }

            let active = if split {
                pane == active_pane
            } else {
                pane == 0
            };
            set_class(&self.panes[pane], "is-active", active);
            let _ = self.panes[pane].set_attribute(
                "aria-hidden",
                if split || pane == 0 { "false" } else { "true" },
            );
        }

        if split {
            let _ = self.pane_close.remove_attribute("hidden");
        } else {
            let _ = self.pane_close.set_attribute("hidden", "");
        }
    }

    fn render_list(&self) {
        let state = self.state.borrow();
        let scroll = self.list.scroll_top();
        self.list.set_inner_html("");

        if state.notes.is_empty() {
            let empty = self.el("div");
            empty.set_class_name("note-list__empty");
            empty.set_text_content(Some("Пока нет заметок"));
            let _ = self.list.append_child(&empty);
        } else {
            for note in &state.notes {
                let item = self.build_note_item(note, &state.session);
                let _ = self.list.append_child(&item);
            }
        }

        let count = state.notes.len();
        self.note_count.set_text_content(Some(&count.to_string()));
        let label = format!(
            "{} {}",
            count,
            plural(count, "заметка", "заметки", "заметок")
        );
        self.sidebar_meta.set_text_content(Some(&label));
        let _ = self.list.set_scroll_top(scroll);
    }

    fn build_note_item(&self, note: &Note, session: &Session) -> Element {
        let active_id = session.pane_note(session.active_pane.min(1));
        let is_active = active_id == Some(note.id.as_str());
        let is_open = session.is_open(&note.id);

        let item = self.el("div");
        let mut classes = String::from("note-item");
        if is_active {
            classes.push_str(" is-active");
        }
        if is_open {
            classes.push_str(" is-open");
        }
        item.set_class_name(&classes);
        let _ = item.set_attribute("data-id", &note.id);
        let _ = item.set_attribute("role", "listitem");

        let open = self.el("button");
        open.set_class_name("note-item__open");
        let _ = open.set_attribute("type", "button");
        let _ = open.set_attribute("data-act", "note-open");
        let _ = open.set_attribute("aria-label", &format!("Открыть «{}»", note.title()));

        let top = self.el("span");
        top.set_class_name("note-item__top");
        let title = self.el("span");
        title.set_class_name("note-item__title");
        title.set_text_content(Some(&note.title()));
        let _ = title.set_attribute("title", &note.title());
        let _ = top.append_child(&title);

        let mut chips_populated = false;
        let chips = self.el("span");
        chips.set_class_name("note-item__chips");
        for pane in 0..2 {
            if session.split && session.pane_note(pane) == Some(note.id.as_str()) {
                let chip = self.el("span");
                let mut chip_class = String::from("chip");
                if session.split && session.active_pane == pane {
                    chip_class.push_str(" is-active");
                }
                chip.set_class_name(&chip_class);
                chip.set_text_content(Some(if pane == 0 { "1" } else { "2" }));
                let _ = chip.set_attribute(
                    "title",
                    if pane == 0 {
                        "Открыта в первой половине"
                    } else {
                        "Открыта во второй половине"
                    },
                );
                let _ = chips.append_child(&chip);
                chips_populated = true;
            }
        }
        if chips_populated {
            let _ = top.append_child(&chips);
        }

        let meta = self.el("span");
        meta.set_class_name("note-item__meta");
        meta.set_text_content(Some(&note.list_meta()));
        let _ = open.append_child(&top);
        let _ = open.append_child(&meta);
        let _ = item.append_child(&open);

        let acts = self.el("div");
        acts.set_class_name("note-item__acts");
        let split_button = self.icon_button(
            "note-split",
            "split",
            "Открыть во второй половине (разделить экран)",
        );
        let delete_button = self.icon_button("note-delete", "trash", "Удалить заметку");
        let _ = acts.append_child(&split_button);
        let _ = acts.append_child(&delete_button);
        let _ = item.append_child(&acts);

        item
    }

    fn icon_button(&self, action: &str, icon_name: &str, label: &str) -> Element {
        let button = self.el("button");
        button.set_class_name("note-item__act");
        let _ = button.set_attribute("type", "button");
        let _ = button.set_attribute("data-act", action);
        let _ = button.set_attribute("title", label);
        let _ = button.set_attribute("aria-label", label);
        button.set_inner_html(&icon(icon_name, ""));
        button
    }

    fn update_list_item(&self, id: &str, title: &str, meta: &str) {
        let selector = format!(".note-item[data-id=\"{id}\"]");
        let Ok(Some(item)) = self.list.query_selector(&selector) else {
            return;
        };
        if let Ok(Some(node)) = item.query_selector(".note-item__title") {
            node.set_text_content(Some(title));
            let _ = node.set_attribute("title", title);
        }
        if let Ok(Some(node)) = item.query_selector(".note-item__meta") {
            node.set_text_content(Some(meta));
        }
        if let Ok(Some(node)) = item.query_selector(".note-item__open") {
            let _ = node.set_attribute("aria-label", &format!("Открыть «{title}»"));
        }
    }

    fn update_document_title(&self) {
        let title = {
            let state = self.state.borrow();
            let pane = state.session.active_pane.min(1);
            state
                .session
                .pane_note(pane)
                .and_then(|id| state.notes.iter().find(|note| note.id == id))
                .map(|note| note.title())
                .unwrap_or_else(|| "заметки".into())
        };
        let full = if title == "Без названия" {
            "plain — заметки".to_string()
        } else {
            format!("{title} — plain")
        };
        self.document.set_title(&full);
    }

    fn update_theme_button(&self) {
        let theme = self.theme.get();
        let (icon_name, label) = match theme {
            Theme::Light => ("moon", "Тёмная тема"),
            Theme::Dark => ("sun", "Светлая тема"),
        };
        self.theme_button.set_inner_html(&icon(icon_name, ""));
        let _ = self.theme_button.set_attribute("title", label);
        let _ = self.theme_button.set_attribute("aria-label", label);
    }

    fn is_narrow(&self) -> bool {
        self.window
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(1024.0)
            <= MOBILE_BREAKPOINT
    }

    fn focus_pane_current(&self) {
        let pane = {
            let state = self.state.borrow();
            if state.session.split {
                state.session.active_pane.min(1)
            } else {
                0
            }
        };
        if let Some(editor) = self.editors.get(pane) {
            let _ = editor.focus();
        }
    }

    fn focus_if_wide(&self) {
        if !self.is_narrow() {
            self.focus_pane_current();
        }
    }

    fn el(&self, tag: &str) -> Element {
        self.document
            .create_element(tag)
            .expect("create_element should not fail")
    }
}

fn set_class(element: &Element, class: &str, enabled: bool) {
    let list = element.class_list();
    if list.contains(class) == enabled {
        return;
    }
    let tokens = js_sys::Array::new();
    tokens.push(&JsValue::from_str(class));
    if enabled {
        let _ = list.add(&tokens);
    } else {
        let _ = list.remove(&tokens);
    }
}

fn query_el(root: &Element, selector: &str) -> Element {
    root.query_selector(selector)
        .ok()
        .flatten()
        .unwrap_or_else(|| panic!("не найден элемент {selector}"))
}

fn query_typed<T: JsCast>(root: &Element, selector: &str) -> T {
    query_el(root, selector)
        .dyn_into::<T>()
        .map_err(|_| ())
        .unwrap_or_else(|_| panic!("неверный тип элемента {selector}"))
}

fn export_filename(title: &str) -> String {
    let slug = util::slugify(title);
    let date = util::date_stamp();
    if slug.is_empty() {
        format!("plain-{date}.md")
    } else {
        format!("plain-{date}-{slug}.md")
    }
}
