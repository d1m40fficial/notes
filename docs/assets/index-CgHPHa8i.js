(function(){const s=document.createElement("link").relList;if(s&&s.supports&&s.supports("modulepreload"))return;for(const n of document.querySelectorAll('link[rel="modulepreload"]'))l(n);new MutationObserver(n=>{for(const a of n)if(a.type==="childList")for(const i of a.addedNodes)i.tagName==="LINK"&&i.rel==="modulepreload"&&l(i)}).observe(document,{childList:!0,subtree:!0});function e(n){const a={};return n.integrity&&(a.integrity=n.integrity),n.referrerPolicy&&(a.referrerPolicy=n.referrerPolicy),n.crossOrigin==="use-credentials"?a.credentials="include":n.crossOrigin==="anonymous"?a.credentials="omit":a.credentials="same-origin",a}function l(n){if(n.ep)return;n.ep=!0;const a=e(n);fetch(n.href,a)}})();const p="plain.note.v1",b="plain.theme.v1",C=240;function k(){const r=new Date,s=e=>String(e).padStart(2,"0");return`${r.getFullYear()}-${s(r.getMonth()+1)}-${s(r.getDate())}`}function T(r){const s=localStorage.getItem(b);(s==="dark"||s==="light")&&(document.documentElement.dataset.theme=s),r.innerHTML=`
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
  `;const e=r.querySelector("#editor"),l=r.querySelector("#saved"),n=r.querySelector("#counts"),a=r.querySelector("#theme");e.value=localStorage.getItem(p)??"";let i=0,u=!1;const m=()=>{const t=e.value,o=t.trim()?t.trim().split(/\s+/).length:0,c=t.length,S=o?Math.max(1,Math.ceil(o/200)):0;n.textContent=`${o} ${o===1?"слово":o>1&&o<5?"слова":"слов"} · ${c} ${c===1?"символ":c>1&&c<5?"символа":"символов"} · ${S} мин`},d=(t=!1)=>{localStorage.setItem(p,e.value),t||(l.textContent="сохранено",window.clearTimeout(i),i=window.setTimeout(()=>{u||(l.textContent="локально")},1600))},E=()=>{l.textContent="сохраняю…",window.clearTimeout(i),i=window.setTimeout(()=>d(),C)},v=()=>{d(!0);const t=new Blob([e.value],{type:"text/markdown;charset=utf-8"}),o=URL.createObjectURL(t),c=document.createElement("a");c.href=o,c.download=`plain-${k()}.md`,c.click(),URL.revokeObjectURL(o),l.textContent="экспорт",window.setTimeout(()=>{u||(l.textContent="локально")},1400)},x=()=>{e.value&&window.confirm("Очистить заметку? Отменить это будет нельзя.")&&(e.value="",localStorage.removeItem(p),m(),l.textContent="очищено",e.focus())},L=()=>{const t=document.documentElement.dataset.theme==="dark"?"light":"dark";document.documentElement.dataset.theme=t,localStorage.setItem(b,t),a.textContent=t==="dark"?"☀":"◐"},f=()=>{m(),E()},h=t=>{if((t.ctrlKey||t.metaKey)&&t.key.toLowerCase()==="s"){t.preventDefault(),v();return}if(t.key==="Tab"){t.preventDefault();const o=e.selectionStart,c=e.selectionEnd;e.value=`${e.value.slice(0,o)}  ${e.value.slice(c)}`,e.selectionStart=e.selectionEnd=o+2,f()}},w=t=>{const o=t.target.closest("[data-act]");o&&(o.dataset.act==="export"&&v(),o.dataset.act==="clear"&&x(),o.dataset.act==="theme"&&L())},g=()=>d(!0);return e.addEventListener("input",f),e.addEventListener("keydown",h),r.addEventListener("click",w),window.addEventListener("beforeunload",g),a.textContent=document.documentElement.dataset.theme==="dark"?"☀":"◐",m(),e.focus(),()=>{u=!0,window.clearTimeout(i),d(!0),e.removeEventListener("input",f),e.removeEventListener("keydown",h),r.removeEventListener("click",w),window.removeEventListener("beforeunload",g)}}const y=document.getElementById("app");y&&T(y);
