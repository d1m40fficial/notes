import { icon } from './icons';
import { el } from './dom';

type ToastKind = 'info' | 'success' | 'error';

export function toast(message: string, kind: ToastKind = 'info', ms = 2800, action?: { label: string; onClick: () => void }): void {
  const host = document.getElementById('toasts');
  if (!host) return;

  const glyph = kind === 'success' ? 'check' : kind === 'error' ? 'warning' : 'sparkles';
  const node = el(`
    <div class="toast toast--${kind}">
      <span class="toast__icon">${icon(glyph)}</span>
      <span class="toast__text">${message}</span>
      ${action ? `<button class="toast__action btn btn--sm" type="button">${action.label}</button>` : ''}
    </div>
  `);

  const close = () => {
    node.classList.add('is-out');
    setTimeout(() => node.remove(), 320);
  };

  node.querySelector<HTMLButtonElement>('.toast__action')?.addEventListener('click', () => {
    action?.onClick();
    close();
  });

  host.appendChild(node);
  const timer = window.setTimeout(close, ms);
  node.addEventListener('mouseenter', () => window.clearTimeout(timer));
  node.addEventListener('mouseleave', () => window.setTimeout(close, 1200));
}
