/* Aura Icon System — review page interactions. */
(function () {
  "use strict";

  var CATEGORY_ORDER = [
    "Навигация",
    "Окно",
    "Трей",
    "Разделы",
    "Движки",
    "Модели",
    "Провайдеры",
    "Фильтры",
    "Действия",
    "Управление",
    "Состояния",
    "Громкость",
    "Звук"
  ];

  // Current Lucide glyphs used by Aura today, for the before/after strip.
  var CURRENT = {
    "Навигация": '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>',
    "Голос": '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="23"></line><line x1="8" y1="23" x2="16" y2="23"></line></svg>',
    "Текст": '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line></svg>',
    "Управление": '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="4" width="20" height="16" rx="2" ry="2"></rect><line x1="6" y1="8" x2="6" y2="8"></line><line x1="10" y1="8" x2="10" y2="8"></line><line x1="14" y1="8" x2="14" y2="8"></line><line x1="18" y1="8" x2="18" y2="8"></line><line x1="6" y1="12" x2="6" y2="12"></line><line x1="18" y1="12" x2="18" y2="12"></line><line x1="7" y1="16" x2="17" y2="16"></line></svg>',
    "История": '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>',
    "О программе": '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>'
  };

  var NEW_FOR_COMPARE = {
    "Навигация": "nav-general",
    "Голос": "nav-speech",
    "Текст": "nav-text",
    "Управление": "nav-hotkeys",
    "История": "nav-history",
    "О программе": "nav-about"
  };

  var state = {
    size: 20,
    category: "Все"
  };

  function hydrate() {
    var nodes = document.querySelectorAll("[data-icon]");
    for (var i = 0; i < nodes.length; i += 1) {
      var node = nodes[i];
      if (node.dataset.hydrated === "1") continue;
      var name = node.getAttribute("data-icon");
      var size = parseInt(node.getAttribute("data-size") || "20", 10);
      var branded = name.indexOf("provider-") === 0 && window.AuraIcons.brandSvg
        ? window.AuraIcons.brandSvg(name, { size: size })
        : "";
      node.innerHTML = branded || window.AuraIcons.svg(name, { size: size });
      node.dataset.hydrated = "1";
    }
  }

  function makeCats() {
    var host = document.getElementById("cat-filter");
    var cats = ["Все"].concat(CATEGORY_ORDER);
    host.innerHTML = "";
    cats.forEach(function (cat) {
      var btn = document.createElement("button");
      btn.type = "button";
      btn.className = "cat-btn" + (cat === state.category ? " active" : "");
      btn.textContent = cat;
      btn.addEventListener("click", function () {
        state.category = cat;
        makeCats();
        renderGrid();
      });
      host.appendChild(btn);
    });
  }

  function renderGrid() {
    var host = document.getElementById("glyph-grid");
    var defs = window.AuraIcons.defs;
    var categories = window.AuraIcons.categories;
    var shown = 0;
    var html = "";

    CATEGORY_ORDER.forEach(function (cat) {
      if (state.category !== "Все" && state.category !== cat) return;
      var names = Object.keys(defs).filter(function (name) {
        return categories[name] === cat;
      });
      if (!names.length) return;
      shown += names.length;
      html += '<div class="glyph-group-title">' + cat + "</div>";
      html += '<div class="glyph-grid">';
      names.forEach(function (name) {
        html +=
          '<div class="glyph-cell">' +
          '<span class="glyph-box">' +
          (name.indexOf("provider-") === 0 && window.AuraIcons.brandSvg
            ? window.AuraIcons.brandSvg(name, { size: state.size })
            : window.AuraIcons.svg(name, { size: state.size })) +
          "</span>" +
          '<span class="glyph-name">' + name + "</span>" +
          "</div>";
      });
      html += "</div>";
    });

    host.innerHTML = html;
    document.getElementById("visible-count").textContent =
      shown + " / " + window.AuraIcons.names.length;
  }

  function renderCompare() {
    var host = document.getElementById("compare-grid");
    var html = "";
    Object.keys(NEW_FOR_COMPARE).forEach(function (label) {
      html +=
        '<div class="compare-cell">' +
        '<div class="compare-side"><span class="compare-label">Сейчас</span><span class="compare-old">' +
        CURRENT[label].replace('width="18" height="18"', 'width="22" height="22"') +
        "</span></div>" +
        '<span class="compare-arrow">→</span>' +
        '<div class="compare-side"><span class="compare-label">Новый</span><span class="compare-new">' +
        window.AuraIcons.svg(NEW_FOR_COMPARE[label], { size: 22 }) +
        "</span></div>" +
        "</div>";
    });
    host.innerHTML = html;
  }

  function bindControls() {
    document.querySelectorAll("[data-size-btn]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        state.size = parseInt(btn.dataset.sizeBtn, 10);
        document.querySelectorAll("[data-size-btn]").forEach(function (b) {
          b.classList.toggle("active", b === btn);
        });
        renderGrid();
      });
    });

    document.querySelectorAll("[data-view-btn]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        document.documentElement.dataset.view = btn.dataset.viewBtn;
        document.querySelectorAll("[data-view-btn]").forEach(function (b) {
          b.classList.toggle("active", b === btn);
        });
      });
    });

    document.querySelectorAll("[data-theme-btn]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        document.documentElement.dataset.theme = btn.dataset.themeBtn;
        document.querySelectorAll("[data-theme-btn]").forEach(function (b) {
          b.classList.toggle("active", b === btn);
        });
      });
    });
  }

  function bindVolume() {
    var stage = document.getElementById("volume-stage");
    var range = document.getElementById("volume-range");
    if (!stage || !range) return;
    var apply = function () {
      var value = Number(range.value);
      var level = value === 0 ? 0 : value < 34 ? 1 : value < 67 ? 2 : 3;
      stage.dataset.level = String(level);
    };
    range.addEventListener("input", apply);
    apply();
  }

  hydrate();
  makeCats();
  renderGrid();
  renderCompare();
  bindControls();
  bindVolume();

  // Review helper: ?only=surfaces|window|panels|glyphs|compare|rules
  // Keeps one block on screen so a 1280-wide headless capture stays readable.
  var params = new URLSearchParams(window.location.search);
  var only = params.get("only");
  if (only) {
    var keep = {
      surfaces: "#surfaces",
      window: "#surfaces",
      panels: "#surfaces",
      volume: "#surfaces",
      engines: "#surfaces",
      providers: "#surfaces",
      glyphs: "#set",
      compare: "#before-after",
      rules: "#rules"
    }[only];
    document.querySelectorAll(".page > *").forEach(function (el) {
      el.style.display = "none";
    });
    var target = document.querySelector(keep);
    if (target) target.style.display = "";
    if (only === "window") {
      var grid = document.querySelector(".surface-grid");
      if (grid) grid.style.display = "none";
    }
    if (only === "panels") {
      var wide = document.querySelector(".panel-wide");
      if (wide) wide.style.display = "none";
    }
    if (only === "volume") {
      var surfaces = document.querySelector("#surfaces");
      if (surfaces) {
        surfaces.querySelectorAll(".block-head, .panel").forEach(function (el) {
          el.style.display = "none";
        });
      }
      var volumePanel = document.querySelector("#volume-panel");
      if (volumePanel) volumePanel.style.display = "";
    }
    if (only === "engines") {
      var surfaces2 = document.querySelector("#surfaces");
      if (surfaces2) {
        surfaces2.querySelectorAll(".block-head, .panel").forEach(function (el) {
          el.style.display = "none";
        });
      }
      var enginePanel = document.querySelector("#engine-panel");
      if (enginePanel) enginePanel.style.display = "";
    }
    if (only === "providers") {
      var surfaces3 = document.querySelector("#surfaces");
      if (surfaces3) {
        surfaces3.querySelectorAll(".block-head, .panel").forEach(function (el) {
          el.style.display = "none";
        });
      }
      var providerPanel = document.querySelector("#provider-panel");
      if (providerPanel) providerPanel.style.display = "";
    }
  }

  var wantedTheme = params.get("theme");
  if (wantedTheme === "light" || wantedTheme === "dark") {
    document.documentElement.dataset.theme = wantedTheme;
    document.querySelectorAll("[data-theme-btn]").forEach(function (b) {
      b.classList.toggle("active", b.dataset.themeBtn === wantedTheme);
    });
  }
})();
