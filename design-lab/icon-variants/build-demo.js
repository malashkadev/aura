import fs from "fs";

const g1 = JSON.parse(fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/group1.json", "utf8"));
const g2 = JSON.parse(fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/group2.json", "utf8"));
const g3 = JSON.parse(fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/group3.json", "utf8"));

const items = [...g1, ...g2, ...g3];

const html = `<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Aura · Варианты иконок настроек</title>
  <style>
    :root {
      --bg: #0d0d0d;
      --surface: #141414;
      --surface-hover: #1c1c1c;
      --border: #262626;
      --border-subtle: #1f1f1f;
      --text: #e6e6e6;
      --text-muted: #888888;
      --text-dim: #555555;
      --accent: #ff4200;
      --accent-dim: rgba(255, 66, 0, 0.12);
      --font: "Onest", "Segoe UI Variable Text", "Segoe UI", system-ui, -apple-system, sans-serif;
    }

    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }

    body {
      background: var(--bg);
      color: var(--text);
      font-family: var(--font);
      font-size: 13px;
      line-height: 1.4;
      padding: 24px 32px 64px 32px;
      -webkit-font-smoothing: antialiased;
    }

    /* Fixed Minimal Bar */
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 24px;
      padding-bottom: 16px;
      border-bottom: 1px solid var(--border);
    }

    .title-group {
      display: flex;
      align-items: baseline;
      gap: 12px;
    }

    h1 {
      font-size: 16px;
      font-weight: 600;
      letter-spacing: -0.01em;
      color: #fff;
    }

    .meta {
      font-size: 12px;
      color: var(--text-muted);
    }

    .stats {
      font-size: 12px;
      font-variant-numeric: tabular-nums;
      color: var(--text-muted);
    }

    /* Table grid structure */
    .table {
      width: 100%;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }

    .table-head {
      display: grid;
      grid-template-columns: 240px 140px 1fr 1fr;
      gap: 12px;
      padding: 8px 14px;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--text-dim);
      font-weight: 600;
    }

    .row {
      display: grid;
      grid-template-columns: 240px 140px 1fr 1fr;
      gap: 12px;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 12px 14px;
      align-items: stretch;
      transition: border-color 0.15s ease;
    }

    .row:hover {
      border-color: #333333;
    }

    /* Column 1: Info */
    .col-info {
      display: flex;
      flex-direction: column;
      justify-content: center;
      gap: 4px;
    }

    .item-id {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      font-size: 11px;
      font-weight: 600;
      color: var(--text-dim);
      font-variant-numeric: tabular-nums;
    }

    .item-title {
      font-size: 13.5px;
      font-weight: 500;
      color: var(--text);
      line-height: 1.35;
    }

    .item-key {
      font-size: 11px;
      font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
      color: var(--text-dim);
    }

    /* Column 2: Current (До) */
    .col-current {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 8px;
      background: #0f0f0f;
      border: 1px solid var(--border-subtle);
      border-radius: 6px;
      padding: 10px;
    }

    .pair-display {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 16px;
    }

    .icon-box {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 5px;
    }

    .icon-label {
      font-size: 9px;
      color: var(--text-dim);
      font-variant-numeric: tabular-nums;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    .glyph {
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--text);
    }

    .glyph svg {
      display: block;
    }

    .col-current .glyph {
      color: #777777;
    }

    /* Column 3 & 4: Variation Cards */
    .var-card {
      position: relative;
      background: #111111;
      border: 1px solid var(--border-subtle);
      border-radius: 6px;
      padding: 10px 12px;
      display: flex;
      flex-direction: column;
      gap: 8px;
      cursor: pointer;
      user-select: none;
      transition: background 0.15s ease, border-color 0.15s ease, transform 0.08s ease;
    }

    .var-card:hover {
      background: var(--surface-hover);
      border-color: #383838;
    }

    .var-card:active {
      transform: scale(0.995);
    }

    .var-card.copied {
      border-color: var(--accent);
      background: var(--accent-dim);
    }

    .card-head {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .badge-code {
      display: inline-flex;
      align-items: center;
      padding: 2px 7px;
      background: #1a1a1a;
      border: 1px solid #333333;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 700;
      color: #fff;
      font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
      letter-spacing: 0.04em;
      transition: background 0.15s, border-color 0.15s, color 0.15s;
    }

    .var-card:hover .badge-code {
      border-color: var(--accent);
      color: var(--accent);
    }

    .var-card.copied .badge-code {
      background: var(--accent);
      border-color: var(--accent);
      color: #fff;
    }

    .copy-hint {
      font-size: 10px;
      color: var(--text-dim);
      opacity: 0;
      transition: opacity 0.15s ease, color 0.15s ease;
    }

    .var-card:hover .copy-hint {
      opacity: 1;
    }

    .var-card.copied .copy-hint {
      opacity: 1;
      color: var(--accent);
      font-weight: 600;
    }

    .card-body {
      display: flex;
      align-items: center;
      gap: 16px;
      padding: 4px 0;
    }

    .card-desc {
      font-size: 11px;
      color: var(--text-muted);
      line-height: 1.4;
      flex: 1;
    }

    /* Toast HUD */
    #toast {
      position: fixed;
      bottom: 24px;
      right: 24px;
      background: #1a1a1a;
      color: #ffffff;
      border: 1px solid var(--accent);
      border-radius: 6px;
      padding: 8px 16px;
      font-size: 12px;
      font-weight: 500;
      box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
      transform: translateY(100px);
      opacity: 0;
      transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease;
      pointer-events: none;
      z-index: 1000;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    #toast.show {
      transform: translateY(0);
      opacity: 1;
    }

    #toast .dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: var(--accent);
    }
  </style>
</head>
<body>

  <header>
    <div class="title-group">
      <h1>Aura · Варианты иконок настроек</h1>
      <span class="meta">24 пункта · 48 векторов · клик по карточке копирует код</span>
    </div>
    <div class="stats">Aura 2.0 Invariants (24px & 18px 1:1)</div>
  </header>

  <div class="table">
    <div class="table-head">
      <div>Пункт настроек</div>
      <div style="text-align: center;">Текущая (До)</div>
      <div>Вариант A</div>
      <div>Вариант B</div>
    </div>

    <div id="list"></div>
  </div>

  <div id="toast"><span class="dot"></span><span id="toast-text"></span></div>

  <script>
    const ITEMS = ${JSON.stringify(items)};

    function renderSvg(body, size) {
      return (
        '<svg width="' + size + '" height="' + size + '" viewBox="0 0 24 24" fill="none" ' +
        'stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" ' +
        'aria-hidden="true" xmlns="http://www.w3.org/2000/svg">' +
        body +
        '</svg>'
      );
    }

    function copyCode(code, cardEl) {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(code).catch(function() {});
      } else {
        const ta = document.createElement("textarea");
        ta.value = code;
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        try { document.execCommand("copy"); } catch (e) {}
        document.body.removeChild(ta);
      }

      cardEl.classList.add("copied");
      const hint = cardEl.querySelector(".copy-hint");
      const oldText = hint ? hint.textContent : "";
      if (hint) hint.textContent = "Скопировано!";

      showToast("Код " + code + " скопирован в буфер");

      setTimeout(function() {
        cardEl.classList.remove("copied");
        if (hint) hint.textContent = oldText;
      }, 1200);
    }

    let toastTimer = null;
    function showToast(msg) {
      const t = document.getElementById("toast");
      const text = document.getElementById("toast-text");
      text.textContent = msg;
      t.classList.add("show");
      clearTimeout(toastTimer);
      toastTimer = setTimeout(function() {
        t.classList.remove("show");
      }, 1600);
    }

    const listContainer = document.getElementById("list");
    let rowsHtml = "";

    for (let i = 0; i < ITEMS.length; i++) {
      const item = ITEMS[i];

      rowsHtml +=
        '<div class="row" data-id="' + item.id + '">' +
          // Col 1: info
          '<div class="col-info">' +
            '<div class="item-id">#' + item.id + '</div>' +
            '<div class="item-title">' + item.title + '</div>' +
            '<div class="item-key">key: ' + item.currentKey + '</div>' +
          '</div>' +

          // Col 2: current (До)
          '<div class="col-current">' +
            '<div class="pair-display">' +
              '<div class="icon-box">' +
                '<div class="glyph">' + renderSvg(item.currentSvg, 24) + '</div>' +
                '<span class="icon-label">24px</span>' +
              '</div>' +
              '<div class="icon-box">' +
                '<div class="glyph">' + renderSvg(item.currentSvg, 18) + '</div>' +
                '<span class="icon-label">18px</span>' +
              '</div>' +
            '</div>' +
          '</div>' +

          // Col 3: Var A
          '<div class="var-card" onclick="copyCode(\\'' + item.codeA + '\\', this)">' +
            '<div class="card-head">' +
              '<span class="badge-code">' + item.codeA + '</span>' +
              '<span class="copy-hint">клик для копирования</span>' +
            '</div>' +
            '<div class="card-body">' +
              '<div class="pair-display">' +
                '<div class="icon-box">' +
                  '<div class="glyph">' + renderSvg(item.varA, 24) + '</div>' +
                  '<span class="icon-label">24px</span>' +
                '</div>' +
                '<div class="icon-box">' +
                  '<div class="glyph">' + renderSvg(item.varA, 18) + '</div>' +
                  '<span class="icon-label">18px</span>' +
                '</div>' +
              '</div>' +
              '<div class="card-desc">' + item.descA + '</div>' +
            '</div>' +
          '</div>' +

          // Col 4: Var B
          '<div class="var-card" onclick="copyCode(\\'' + item.codeB + '\\', this)">' +
            '<div class="card-head">' +
              '<span class="badge-code">' + item.codeB + '</span>' +
              '<span class="copy-hint">клик для копирования</span>' +
            '</div>' +
            '<div class="card-body">' +
              '<div class="pair-display">' +
                '<div class="icon-box">' +
                  '<div class="glyph">' + renderSvg(item.varB, 24) + '</div>' +
                  '<span class="icon-label">24px</span>' +
                '</div>' +
                '<div class="icon-box">' +
                  '<div class="glyph">' + renderSvg(item.varB, 18) + '</div>' +
                  '<span class="icon-label">18px</span>' +
                '</div>' +
              '</div>' +
              '<div class="card-desc">' + item.descB + '</div>' +
            '</div>' +
          '</div>' +
        '</div>';
    }

    listContainer.innerHTML = rowsHtml;
  </script>
</body>
</html>
`;

fs.writeFileSync("G:/Aura/2.0/design-lab/icon-variants/settings-icons.html", html, "utf8");
console.log("Written settings-icons.html successfully! Size:", html.length, "bytes");
