const NOTE_KEY = 'plain.note.v1';
const THEME_KEY = 'plain.theme.v1';
const SAVE_DELAY = 240;

function fmtDate(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function mount(root: HTMLElement): () => void {
  const storedTheme = localStorage.getItem(THEME_KEY);
  if (storedTheme === 'dark' || storedTheme === 'light') document.documentElement.dataset.theme = storedTheme;

  root.innerHTML = `
    <div class="page">
      <header class="bar">
        <span class="wordmark">plain<span>.</span></span>
        <span class="bar__saved" id="saved" aria-live="polite">локально</span>
        <nav class="bar__actions" aria-label="Действия">
          <button type="button" data-act="export" title="Скачать .md (Ctrl+S)">export</button>
          <button type="button" data-act="clear" title="Очистить заметку">clear</button>
          <button type="button" data-act="theme" id="theme" title="Сменить тему">◐</button>
        </nav>
      </header>
      <section class="paper">
        <label class="sr-only" for="editor">Заметка</label>
        <textarea id="editor" spellcheck="true" placeholder="Начните печатать. Всё сохраняется в этом браузере."></textarea>
      </section>
      <footer class="status">
        <span id="counts">0 слов · 0 символов</span>
        <span class="status__hint">autosave · ctrl+s export</span>
      </footer>
    </div>
  `;

  const editor = root.querySelector<HTMLTextAreaElement>('#editor')!;
  const saved = root.querySelector<HTMLElement>('#saved')!;
  const counts = root.querySelector<HTMLElement>('#counts')!;
  const themeButton = root.querySelector<HTMLButtonElement>('#theme')!;

  editor.value = localStorage.getItem(NOTE_KEY) ?? '';
  let timer = 0;
  let disposed = false;

  const updateCounts = () => {
    const text = editor.value;
    const words = text.trim() ? text.trim().split(/\s+/).length : 0;
    const chars = text.length;
    const minutes = words ? Math.max(1, Math.ceil(words / 200)) : 0;
    counts.textContent = `${words} ${words === 1 ? 'слово' : words > 1 && words < 5 ? 'слова' : 'слов'} · ${chars} ${chars === 1 ? 'символ' : chars > 1 && chars < 5 ? 'символа' : 'символов'} · ${minutes} мин`;
  };

  const save = (silent = false) => {
    localStorage.setItem(NOTE_KEY, editor.value);
    if (!silent) {
      saved.textContent = 'сохранено';
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        if (!disposed) saved.textContent = 'локально';
      }, 1600);
    }
  };

  const scheduleSave = () => {
    saved.textContent = 'сохраняю…';
    window.clearTimeout(timer);
    timer = window.setTimeout(() => save(), SAVE_DELAY);
  };

  const exportNote = () => {
    save(true);
    const blob = new Blob([editor.value], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `plain-${fmtDate()}.md`;
    a.click();
    URL.revokeObjectURL(url);
    saved.textContent = 'экспорт';
    window.setTimeout(() => {
      if (!disposed) saved.textContent = 'локально';
    }, 1400);
  };

  const clearNote = () => {
    if (!editor.value) return;
    if (!window.confirm('Очистить заметку? Отменить это будет нельзя.')) return;
    editor.value = '';
    localStorage.removeItem(NOTE_KEY);
    updateCounts();
    saved.textContent = 'очищено';
    editor.focus();
  };

  const toggleTheme = () => {
    const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
    document.documentElement.dataset.theme = next;
    localStorage.setItem(THEME_KEY, next);
    themeButton.textContent = next === 'dark' ? '☀' : '◐';
  };

  const onInput = () => {
    updateCounts();
    scheduleSave();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
      event.preventDefault();
      exportNote();
      return;
    }
    if (event.key === 'Tab') {
      event.preventDefault();
      const start = editor.selectionStart;
      const end = editor.selectionEnd;
      editor.value = `${editor.value.slice(0, start)}  ${editor.value.slice(end)}`;
      editor.selectionStart = editor.selectionEnd = start + 2;
      onInput();
    }
  };

  const onClick = (event: Event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>('[data-act]');
    if (!button) return;
    if (button.dataset.act === 'export') exportNote();
    if (button.dataset.act === 'clear') clearNote();
    if (button.dataset.act === 'theme') toggleTheme();
  };

  const onBeforeUnload = () => save(true);

  editor.addEventListener('input', onInput);
  editor.addEventListener('keydown', onKeyDown);
  root.addEventListener('click', onClick);
  window.addEventListener('beforeunload', onBeforeUnload);

  themeButton.textContent = document.documentElement.dataset.theme === 'dark' ? '☀' : '◐';
  updateCounts();
  editor.focus();

  return () => {
    disposed = true;
    window.clearTimeout(timer);
    save(true);
    editor.removeEventListener('input', onInput);
    editor.removeEventListener('keydown', onKeyDown);
    root.removeEventListener('click', onClick);
    window.removeEventListener('beforeunload', onBeforeUnload);
  };
}
