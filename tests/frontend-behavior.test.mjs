import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

const sourceCache = new Map();
const transpileCache = new Map();

function readProjectFile(relPath) {
  if (!sourceCache.has(relPath)) {
    sourceCache.set(relPath, readFileSync(new URL(`../${relPath}`, import.meta.url), "utf8"));
  }
  return sourceCache.get(relPath);
}

function transpileSrcModule(fileName) {
  if (!transpileCache.has(fileName)) {
    const raw = readProjectFile(`src/${fileName}`);
    const { outputText } = ts.transpileModule(raw, {
      fileName,
      compilerOptions: {
        module: ts.ModuleKind.CommonJS,
        target: ts.ScriptTarget.ES2022,
      },
    });
    transpileCache.set(fileName, outputText);
  }
  return transpileCache.get(fileName);
}

async function flushAsync(turns = 12) {
  for (let i = 0; i < turns; i += 1) {
    await Promise.resolve();
  }
}

// ---------------------------------------------------------------------------
// Minimal deterministic W3C DOM + Virtual Clock + Tauri Bridge for node:test
// ---------------------------------------------------------------------------

const VOID_TAGS = new Set([
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "param",
  "source",
  "track",
  "wbr",
]);

function decodeEntities(str) {
  return String(str)
    .replaceAll("&quot;", '"')
    .replaceAll("&#039;", "'")
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

function camelToKebab(prop) {
  if (prop.startsWith("--")) return prop;
  return prop.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`);
}

function dataKeyToAttr(key) {
  return "data-" + key.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`);
}

function attrToDataKey(attr) {
  return attr.slice(5).replace(/-([a-z])/g, (_, ch) => ch.toUpperCase());
}

class MockEvent {
  constructor(type, init = {}) {
    this.type = String(type);
    this.bubbles = Boolean(init.bubbles);
    this.cancelable = Boolean(init.cancelable);
    this.button = init.button ?? 0;
    this.key = init.key ?? "";
    this.code = init.code ?? "";
    this.ctrlKey = Boolean(init.ctrlKey);
    this.altKey = Boolean(init.altKey);
    this.shiftKey = Boolean(init.shiftKey);
    this.metaKey = Boolean(init.metaKey);
    this.propertyName = init.propertyName ?? "";
    this.relatedTarget = init.relatedTarget ?? null;
    this.detail = init.detail ?? null;
    this.defaultPrevented = false;
    this._stopped = false;
    this.target = null;
    this.currentTarget = null;
  }

  preventDefault() {
    this.defaultPrevented = true;
  }

  stopPropagation() {
    this._stopped = true;
  }
}

class MockEventTarget {
  constructor() {
    this._listeners = new Map();
  }

  addEventListener(type, listener, options) {
    if (typeof listener !== "function") return;
    if (!this._listeners.has(type)) {
      this._listeners.set(type, []);
    }
    const once = typeof options === "object" && options !== null && Boolean(options.once);
    const list = this._listeners.get(type);
    if (!list.some((entry) => entry.listener === listener)) {
      list.push({ listener, once });
    }
  }

  removeEventListener(type, listener) {
    const list = this._listeners.get(type);
    if (!list) return;
    const idx = list.findIndex((entry) => entry.listener === listener);
    if (idx !== -1) list.splice(idx, 1);
  }

  dispatchEvent(event) {
    if (!event.target) {
      event.target = this;
    }
    let current = this;
    while (current) {
      event.currentTarget = current;
      const entries = [...(current._listeners.get(event.type) || [])];
      for (const entry of entries) {
        if (entry.once) {
          current.removeEventListener(event.type, entry.listener);
        }
        entry.listener.call(current, event);
      }
      if (!event.bubbles || event._stopped) {
        break;
      }
      current = current.parentNode || null;
    }
    return !event.defaultPrevented;
  }
}

class MockNode extends MockEventTarget {
  constructor(nodeType, ownerDocument = null) {
    super();
    this.nodeType = nodeType;
    this.ownerDocument = ownerDocument;
    this.parentNode = null;
    this.childNodes = [];
  }

  get parentElement() {
    return this.parentNode && this.parentNode.nodeType === 1 ? this.parentNode : null;
  }

  get firstChild() {
    return this.childNodes[0] || null;
  }

  get lastChild() {
    return this.childNodes[this.childNodes.length - 1] || null;
  }

  get previousSibling() {
    if (!this.parentNode) return null;
    const idx = this.parentNode.childNodes.indexOf(this);
    return idx > 0 ? this.parentNode.childNodes[idx - 1] : null;
  }

  get nextSibling() {
    if (!this.parentNode) return null;
    const idx = this.parentNode.childNodes.indexOf(this);
    return idx >= 0 && idx < this.parentNode.childNodes.length - 1
      ? this.parentNode.childNodes[idx + 1]
      : null;
  }

  get previousElementSibling() {
    let cur = this.previousSibling;
    while (cur && cur.nodeType !== 1) cur = cur.previousSibling;
    return cur;
  }

  get nextElementSibling() {
    let cur = this.nextSibling;
    while (cur && cur.nodeType !== 1) cur = cur.nextSibling;
    return cur;
  }

  get children() {
    return this.childNodes.filter((n) => n.nodeType === 1);
  }

  _coerceNode(item) {
    if (item instanceof MockNode) return item;
    const doc = this.nodeType === 9 ? this : this.ownerDocument;
    return new MockTextNode(String(item ?? ""), doc);
  }

  appendChild(node) {
    return this.insertBefore(node, null);
  }

  insertBefore(node, refNode) {
    if (node.nodeType === 11) {
      const items = [...node.childNodes];
      node.childNodes = [];
      for (const child of items) {
        child.parentNode = null;
        this.insertBefore(child, refNode);
      }
      return node;
    }
    if (node.parentNode) {
      node.parentNode.removeChild(node);
    }
    node.parentNode = this;
    if (!refNode) {
      this.childNodes.push(node);
    } else {
      const idx = this.childNodes.indexOf(refNode);
      if (idx === -1) {
        this.childNodes.push(node);
      } else {
        this.childNodes.splice(idx, 0, node);
      }
    }
    return node;
  }

  removeChild(node) {
    const idx = this.childNodes.indexOf(node);
    if (idx !== -1) {
      this.childNodes.splice(idx, 1);
      node.parentNode = null;
    }
    return node;
  }

  remove() {
    if (this.parentNode) {
      this.parentNode.removeChild(this);
    }
  }

  append(...items) {
    const snapshot = [...items];
    for (const item of snapshot) {
      this.appendChild(this._coerceNode(item));
    }
  }

  prepend(...items) {
    const snapshot = [...items];
    const ref = this.firstChild;
    for (const item of snapshot) {
      this.insertBefore(this._coerceNode(item), ref);
    }
  }

  after(...items) {
    if (!this.parentNode) return;
    const snapshot = [...items];
    const ref = this.nextSibling;
    for (const item of snapshot) {
      this.parentNode.insertBefore(this._coerceNode(item), ref);
    }
  }

  replaceChildren(...items) {
    for (const child of [...this.childNodes]) {
      child.parentNode = null;
    }
    this.childNodes = [];
    this.append(...items);
  }

  contains(other) {
    let cur = other;
    while (cur) {
      if (cur === this) return true;
      cur = cur.parentNode;
    }
    return false;
  }
}

MockNode.ELEMENT_NODE = 1;
MockNode.TEXT_NODE = 3;
MockNode.DOCUMENT_NODE = 9;
MockNode.DOCUMENT_FRAGMENT_NODE = 11;

class MockTextNode extends MockNode {
  constructor(data, ownerDocument) {
    super(3, ownerDocument);
    this.nodeValue = String(data);
  }

  get textContent() {
    return this.nodeValue;
  }

  set textContent(val) {
    this.nodeValue = String(val ?? "");
  }
}

class MockDocumentFragment extends MockNode {
  constructor(ownerDocument) {
    super(11, ownerDocument);
  }

  querySelector(selector) {
    return queryAllFromRoot(this, selector)[0] || null;
  }

  querySelectorAll(selector) {
    return queryAllFromRoot(this, selector);
  }
}

function createStyleProxy(element) {
  const map = new Map();
  const syncAttr = () => {
    if (map.size === 0) {
      element._attrs.delete("style");
      return;
    }
    const serialized = [...map.entries()].map(([k, v]) => `${k}: ${v};`).join(" ");
    element._attrs.set("style", serialized);
  };
  const api = {
    setProperty(prop, val) {
      const key = camelToKebab(String(prop));
      if (val === "" || val === null || val === undefined) {
        map.delete(key);
      } else {
        map.set(key, String(val));
      }
      syncAttr();
    },
    getPropertyValue(prop) {
      return map.get(camelToKebab(String(prop))) || "";
    },
    removeProperty(prop) {
      const key = camelToKebab(String(prop));
      const old = map.get(key) || "";
      map.delete(key);
      syncAttr();
      return old;
    },
    _loadFromCssText(cssText) {
      map.clear();
      for (const part of String(cssText || "").split(";")) {
        const colon = part.indexOf(":");
        if (colon === -1) continue;
        const k = part.slice(0, colon).trim();
        const v = part.slice(colon + 1).trim();
        if (k) map.set(k, v);
      }
    },
  };
  return new Proxy(api, {
    get(target, prop) {
      if (prop in target || typeof prop === "symbol") {
        return target[prop];
      }
      return map.get(camelToKebab(prop)) || "";
    },
    set(target, prop, value) {
      if (prop in target || typeof prop === "symbol") {
        target[prop] = value;
        return true;
      }
      api.setProperty(prop, value);
      return true;
    },
  });
}

class MockElement extends MockNode {
  constructor(tagName, ownerDocument) {
    super(1, ownerDocument);
    this.tagName = String(tagName).toUpperCase();
    this._attrs = new Map();
    this._classes = new Set();
    this.style = createStyleProxy(this);
    this._value = undefined;
    this._checked = false;
    this.inert = false;
    this.offsetWidth = 120;
    this.offsetHeight = 32;
    this.offsetLeft = 0;
    this.offsetTop = 0;

    const self = this;
    this.classList = {
      add(...tokens) {
        for (const t of tokens) {
          if (t) self._classes.add(String(t));
        }
        self._syncClassAttr();
      },
      remove(...tokens) {
        for (const t of tokens) {
          self._classes.delete(String(t));
        }
        self._syncClassAttr();
      },
      toggle(token, force) {
        const has = self._classes.has(token);
        const next = force !== undefined ? Boolean(force) : !has;
        if (next) self._classes.add(token);
        else self._classes.delete(token);
        self._syncClassAttr();
        return next;
      },
      contains(token) {
        return self._classes.has(token);
      },
      [Symbol.iterator]() {
        return self._classes[Symbol.iterator]();
      },
    };

    this.dataset = new Proxy(
      {},
      {
        get(_, prop) {
          if (typeof prop !== "string") return undefined;
          return self._attrs.get(dataKeyToAttr(prop));
        },
        set(_, prop, val) {
          if (typeof prop === "string") {
            self.setAttribute(dataKeyToAttr(prop), String(val));
          }
          return true;
        },
        deleteProperty(_, prop) {
          if (typeof prop === "string") {
            self.removeAttribute(dataKeyToAttr(prop));
          }
          return true;
        },
        ownKeys() {
          return [...self._attrs.keys()]
            .filter((k) => k.startsWith("data-"))
            .map(attrToDataKey);
        },
        getOwnPropertyDescriptor(_, prop) {
          if (typeof prop === "string" && self._attrs.has(dataKeyToAttr(prop))) {
            return { enumerable: true, configurable: true };
          }
          return undefined;
        },
      },
    );
  }

  _syncClassAttr() {
    if (this._classes.size === 0) {
      this._attrs.delete("class");
    } else {
      this._attrs.set("class", [...this._classes].join(" "));
    }
  }

  get id() {
    return this._attrs.get("id") || "";
  }

  set id(val) {
    if (val === "" || val === null || val === undefined) {
      this._attrs.delete("id");
    } else {
      this._attrs.set("id", String(val));
    }
  }

  get className() {
    return [...this._classes].join(" ");
  }

  set className(val) {
    this._classes.clear();
    for (const token of String(val || "").split(/\s+/)) {
      if (token) this._classes.add(token);
    }
    this._syncClassAttr();
  }

  get lang() {
    return this._attrs.get("lang") || "";
  }

  set lang(val) {
    this.setAttribute("lang", String(val ?? ""));
  }

  get title() {
    return this._attrs.get("title") || "";
  }

  set title(val) {
    this.setAttribute("title", String(val ?? ""));
  }

  get hidden() {
    return this._attrs.has("hidden");
  }

  set hidden(val) {
    if (val) this._attrs.set("hidden", "");
    else this._attrs.delete("hidden");
  }

  get disabled() {
    return this._attrs.has("disabled");
  }

  set disabled(val) {
    if (val) this._attrs.set("disabled", "");
    else this._attrs.delete("disabled");
  }

  get tabIndex() {
    const raw = this._attrs.get("tabindex");
    return raw !== undefined ? Number(raw) : 0;
  }

  set tabIndex(val) {
    this.setAttribute("tabindex", String(val));
  }

  get type() {
    if (this.tagName === "SELECT") return "select-one";
    if (this.tagName === "TEXTAREA") return "textarea";
    if (this.tagName === "BUTTON") return this._attrs.get("type") || "submit";
    return (this._attrs.get("type") || "text").toLowerCase();
  }

  set type(val) {
    this.setAttribute("type", String(val));
  }

  get placeholder() {
    return this._attrs.get("placeholder") || "";
  }

  set placeholder(val) {
    this.setAttribute("placeholder", String(val ?? ""));
  }

  get href() {
    return this._attrs.get("href") || "";
  }

  set href(val) {
    this.setAttribute("href", String(val ?? ""));
  }

  get checked() {
    return Boolean(this._checked);
  }

  set checked(val) {
    const next = Boolean(val);
    this._checked = next;
    if (next && this.tagName === "INPUT" && this.type === "radio" && this.ownerDocument) {
      const groupName = this._attrs.get("name");
      if (groupName) {
        for (const el of queryAllFromRoot(this.ownerDocument, "input")) {
          if (el !== this && el.type === "radio" && el.getAttribute("name") === groupName) {
            el._checked = false;
          }
        }
      }
    }
  }

  get options() {
    if (this.tagName !== "SELECT") return [];
    return queryAllFromRoot(this, "option");
  }

  get selectedOptions() {
    if (this.tagName !== "SELECT") return [];
    const opts = this.options;
    if (opts.length === 0) return [];
    const current = this.value;
    const match = opts.find((o) => o.value === current);
    return match ? [match] : [opts[0]];
  }

  get value() {
    if (this.tagName === "OPTION") {
      return this._attrs.has("value") ? this._attrs.get("value") : this.textContent.trim();
    }
    if (this.tagName === "SELECT") {
      const opts = this.options;
      if (opts.length === 0) return "";
      if (this._value !== undefined && opts.some((o) => o.value === this._value)) {
        return this._value;
      }
      const marked = opts.find((o) => o.hasAttribute("selected"));
      return marked ? marked.value : opts[0].value;
    }
    if (this.tagName === "TEXTAREA") {
      return this._value !== undefined ? this._value : this.textContent;
    }
    if (this._value !== undefined) {
      return this._value;
    }
    return this._attrs.get("value") || "";
  }

  set value(val) {
    const strVal = String(val ?? "");
    if (this.tagName === "SELECT") {
      const opts = this.options;
      if (opts.some((o) => o.value === strVal)) {
        this._value = strVal;
      } else if (opts.length === 0) {
        this._value = "";
      }
      return;
    }
    this._value = strVal;
  }

  get textContent() {
    let out = "";
    const walk = (node) => {
      for (const child of node.childNodes) {
        if (child.nodeType === 3) out += child.nodeValue;
        else if (child.nodeType === 1 || child.nodeType === 11) walk(child);
      }
    };
    walk(this);
    return out;
  }

  set textContent(val) {
    for (const child of [...this.childNodes]) {
      child.parentNode = null;
    }
    this.childNodes = [];
    const str = String(val ?? "");
    if (str.length > 0) {
      this.appendChild(new MockTextNode(str, this.ownerDocument));
    }
  }

  get innerHTML() {
    return this.textContent;
  }

  set innerHTML(markup) {
    for (const child of [...this.childNodes]) {
      child.parentNode = null;
    }
    this.childNodes = [];
    if (markup) {
      const frag = parseHtmlFragment(String(markup), this.ownerDocument);
      this.appendChild(frag);
    }
  }

  get offsetParent() {
    let cur = this;
    while (cur && cur.nodeType === 1) {
      if (cur.hidden || cur.style.display === "none") {
        return null;
      }
      cur = cur.parentNode;
    }
    return this.ownerDocument ? this.ownerDocument.body : {};
  }

  setAttribute(name, value) {
    const key = String(name).toLowerCase();
    const strVal = String(value);
    this._attrs.set(key, strVal);
    if (key === "class") {
      this._classes.clear();
      for (const token of strVal.split(/\s+/)) {
        if (token) this._classes.add(token);
      }
    } else if (key === "style") {
      this.style._loadFromCssText(strVal);
    } else if (key === "checked") {
      this._checked = true;
    }
  }

  getAttribute(name) {
    const key = String(name).toLowerCase();
    return this._attrs.has(key) ? this._attrs.get(key) : null;
  }

  hasAttribute(name) {
    return this._attrs.has(String(name).toLowerCase());
  }

  removeAttribute(name) {
    const key = String(name).toLowerCase();
    this._attrs.delete(key);
    if (key === "class") this._classes.clear();
    if (key === "style") this.style._loadFromCssText("");
  }

  getBoundingClientRect() {
    return {
      left: 0,
      top: 0,
      width: this.offsetWidth,
      height: this.offsetHeight,
      right: this.offsetWidth,
      bottom: this.offsetHeight,
    };
  }

  scrollIntoView() {}

  focus() {
    if (this.ownerDocument) {
      this.ownerDocument.activeElement = this;
    }
  }

  blur() {
    if (this.ownerDocument && this.ownerDocument.activeElement === this) {
      this.ownerDocument.activeElement = this.ownerDocument.body;
    }
  }

  click() {
    if (this.disabled) return;
    if (this.tagName === "INPUT" && this.type === "checkbox") {
      this.checked = !this.checked;
      this.dispatchEvent(new MockEvent("click", { bubbles: true, cancelable: true }));
      this.dispatchEvent(new MockEvent("change", { bubbles: true }));
      return;
    }
    if (this.tagName === "INPUT" && this.type === "radio") {
      const wasChecked = this.checked;
      this.checked = true;
      this.dispatchEvent(new MockEvent("click", { bubbles: true, cancelable: true }));
      if (!wasChecked) {
        this.dispatchEvent(new MockEvent("change", { bubbles: true }));
      }
      return;
    }
    this.dispatchEvent(new MockEvent("click", { bubbles: true, cancelable: true }));
  }

  matches(selector) {
    return matchesSelectorList(this, selector, null);
  }

  closest(selector) {
    let cur = this;
    while (cur && cur.nodeType === 1) {
      if (matchesSelectorList(cur, selector, null)) return cur;
      cur = cur.parentNode;
    }
    return null;
  }

  querySelector(selector) {
    return queryAllFromRoot(this, selector)[0] || null;
  }

  querySelectorAll(selector) {
    return queryAllFromRoot(this, selector);
  }
}

class MockHTMLElement extends MockElement {}

class MockHTMLDialogElement extends MockHTMLElement {
  constructor(ownerDocument) {
    super("dialog", ownerDocument);
    this.open = false;
  }

  showModal() {
    this.open = true;
  }

  close() {
    this.open = false;
  }
}

class MockDocument extends MockNode {
  constructor() {
    super(9, null);
    this.title = "";
    this.hidden = false;
    this.fonts = { ready: Promise.resolve() };
    this.documentElement = this.createElement("html");
    this.head = this.createElement("head");
    this.body = this.createElement("body");
    this.appendChild(this.documentElement);
    this.documentElement.appendChild(this.head);
    this.documentElement.appendChild(this.body);
    this.activeElement = this.body;
  }

  hasFocus() {
    return !this.hidden;
  }

  createElement(tagName) {
    const lower = String(tagName).toLowerCase();
    if (lower === "dialog") {
      return new MockHTMLDialogElement(this);
    }
    return new MockHTMLElement(lower, this);
  }

  createTextNode(text) {
    return new MockTextNode(text, this);
  }

  createDocumentFragment() {
    return new MockDocumentFragment(this);
  }

  getElementById(id) {
    const target = String(id);
    let found = null;
    const walk = (node) => {
      for (const child of node.childNodes) {
        if (found) return;
        if (child.nodeType === 1) {
          if (child.id === target) {
            found = child;
            return;
          }
          walk(child);
        }
      }
    };
    walk(this);
    return found;
  }

  querySelector(selector) {
    return queryAllFromRoot(this, selector)[0] || null;
  }

  querySelectorAll(selector) {
    return queryAllFromRoot(this, selector);
  }
}

// ---------------------------------------------------------------------------
// CSS Selector Matcher (supports ',', ' ', '>', :scope, #id, .cls, [attr], :checked, :not)
// ---------------------------------------------------------------------------

function splitTopLevel(input, delimiterChar) {
  const parts = [];
  let current = "";
  let parenDepth = 0;
  let bracketDepth = 0;
  let quote = null;
  for (let i = 0; i < input.length; i += 1) {
    const ch = input[i];
    if (quote) {
      current += ch;
      if (ch === quote && input[i - 1] !== "\\") quote = null;
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      current += ch;
      continue;
    }
    if (ch === "[") bracketDepth += 1;
    else if (ch === "]") bracketDepth = Math.max(0, bracketDepth - 1);
    else if (ch === "(") parenDepth += 1;
    else if (ch === ")") parenDepth = Math.max(0, parenDepth - 1);

    if (ch === delimiterChar && parenDepth === 0 && bracketDepth === 0) {
      parts.push(current.trim());
      current = "";
    } else {
      current += ch;
    }
  }
  if (current.trim()) parts.push(current.trim());
  return parts;
}

function tokenizeChain(selector) {
  // Returns alternating [compound, combinator, compound, ...], where combinator is ' ' or '>'
  const tokens = [];
  let buf = "";
  let parenDepth = 0;
  let bracketDepth = 0;
  let quote = null;
  for (let i = 0; i < selector.length; i += 1) {
    const ch = selector[i];
    if (quote) {
      buf += ch;
      if (ch === quote && selector[i - 1] !== "\\") quote = null;
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      buf += ch;
      continue;
    }
    if (ch === "[") bracketDepth += 1;
    else if (ch === "]") bracketDepth = Math.max(0, bracketDepth - 1);
    else if (ch === "(") parenDepth += 1;
    else if (ch === ")") parenDepth = Math.max(0, parenDepth - 1);

    if (parenDepth === 0 && bracketDepth === 0 && (ch === ">" || /\s/.test(ch))) {
      if (buf) {
        tokens.push(buf);
        buf = "";
      }
      let comb = ch === ">" ? ">" : " ";
      while (i + 1 < selector.length && (selector[i + 1] === ">" || /\s/.test(selector[i + 1]))) {
        if (selector[i + 1] === ">") comb = ">";
        i += 1;
      }
      if (tokens.length > 0) {
        tokens.push(comb);
      }
    } else {
      buf += ch;
    }
  }
  if (buf) tokens.push(buf);
  return tokens;
}

function matchesCompound(el, compound, scopeRoot) {
  if (!el || el.nodeType !== 1) return false;
  let rest = compound.trim();
  if (rest === "*") return true;

  // Handle :not(...) pseudo-classes
  while (rest.includes(":not(")) {
    const idx = rest.indexOf(":not(");
    let depth = 1;
    let end = idx + 5;
    while (end < rest.length && depth > 0) {
      if (rest[end] === "(") depth += 1;
      else if (rest[end] === ")") depth -= 1;
      end += 1;
    }
    const inner = rest.slice(idx + 5, end - 1);
    if (matchesSelectorList(el, inner, scopeRoot)) return false;
    rest = rest.slice(0, idx) + rest.slice(end);
  }

  if (rest.includes(":scope")) {
    if (scopeRoot && el !== scopeRoot) return false;
    rest = rest.replaceAll(":scope", "");
  }

  if (rest.includes(":checked")) {
    if (!el.checked) return false;
    rest = rest.replaceAll(":checked", "");
  }

  // Extract [attr...] blocks
  const attrBlocks = [];
  rest = rest.replace(/\[([^\]]+)\]/g, (_, inner) => {
    attrBlocks.push(inner.trim());
    return "";
  });

  for (const block of attrBlocks) {
    const m = block.match(/^([a-zA-Z0-9_:-]+)\s*(?:(\^=|\*=|=)\s*(?:"([^"]*)"|'([^']*)'|([^\s"']+)))?$/);
    if (!m) return false;
    const attrName = m[1].toLowerCase();
    const op = m[2];
    const expected = m[3] ?? m[4] ?? m[5] ?? "";
    if (!op) {
      if (!el.hasAttribute(attrName)) return false;
    } else {
      const actual = el.getAttribute(attrName);
      if (actual === null) return false;
      if (op === "=" && actual !== expected) return false;
      if (op === "^=" && !actual.startsWith(expected)) return false;
      if (op === "*=" && !actual.includes(expected)) return false;
    }
  }

  // Extract #id and .class tokens and optional leading tag
  let i = 0;
  let tag = "";
  while (i < rest.length && rest[i] !== "#" && rest[i] !== ".") {
    tag += rest[i];
    i += 1;
  }
  if (tag && tag !== "*" && el.tagName !== tag.toUpperCase()) {
    return false;
  }

  while (i < rest.length) {
    const kind = rest[i];
    i += 1;
    let name = "";
    while (i < rest.length && rest[i] !== "#" && rest[i] !== ".") {
      name += rest[i];
      i += 1;
    }
    if (kind === "#" && el.id !== name) return false;
    if (kind === "." && !el.classList.contains(name)) return false;
  }

  return true;
}

function matchesChainAt(el, chain, idx, scopeRoot) {
  if (!matchesCompound(el, chain[idx], scopeRoot)) return false;
  if (idx === 0) return true;
  const comb = chain[idx - 1];
  if (comb === ">") {
    return Boolean(el.parentElement && matchesChainAt(el.parentElement, chain, idx - 2, scopeRoot));
  }
  let ancestor = el.parentElement;
  while (ancestor) {
    if (matchesChainAt(ancestor, chain, idx - 2, scopeRoot)) return true;
    ancestor = ancestor.parentElement;
  }
  return false;
}

function matchesSelectorList(el, selectorList, scopeRoot) {
  const branches = splitTopLevel(selectorList, ",");
  for (const branch of branches) {
    const chain = tokenizeChain(branch);
    if (chain.length > 0 && matchesChainAt(el, chain, chain.length - 1, scopeRoot)) {
      return true;
    }
  }
  return false;
}

function queryAllFromRoot(root, selectorList) {
  const scopeRoot = root.nodeType === 1 ? root : root.documentElement || null;
  const results = [];
  const walk = (node) => {
    for (const child of node.childNodes) {
      if (child.nodeType === 1) {
        if (matchesSelectorList(child, selectorList, scopeRoot)) {
          results.push(child);
        }
        walk(child);
      }
    }
  };
  walk(root);
  return results;
}

// ---------------------------------------------------------------------------
// Lightweight HTML Parser -> MockDocument / MockDocumentFragment
// ---------------------------------------------------------------------------

function parseAttributesInto(attrText, el) {
  const attrRegex = /([a-zA-Z_:][-a-zA-Z0-9_:.]*)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;
  let m;
  while ((m = attrRegex.exec(attrText)) !== null) {
    const name = m[1];
    const rawVal = m[2] ?? m[3] ?? m[4] ?? "";
    el.setAttribute(name, decodeEntities(rawVal));
  }
}

function parseHtmlTokens(markup, rootContainer, doc) {
  const cleaned = markup
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/<!DOCTYPE[^>]*>/gi, "")
    .replace(/<style\b[\s\S]*?<\/style>/gi, "")
    .replace(/<script\b[\s\S]*?<\/script>/gi, "");

  const stack = [rootContainer];
  const tagRegex = /<\/?([a-zA-Z][a-zA-Z0-9:-]*)(\s[^<>]*?)?\s*(\/?)>/g;
  let lastIndex = 0;
  let m;

  while ((m = tagRegex.exec(cleaned)) !== null) {
    if (m.index > lastIndex) {
      const textChunk = decodeEntities(cleaned.slice(lastIndex, m.index));
      if (textChunk) {
        stack[stack.length - 1].appendChild(new MockTextNode(textChunk, doc));
      }
    }
    lastIndex = tagRegex.lastIndex;

    const full = m[0];
    const tagLower = m[1].toLowerCase();
    const attrPart = m[2] || "";
    const explicitSelfClose = m[3] === "/";
    const isClosing = full.startsWith("</");

    if (tagLower === "html") {
      if (!isClosing && doc && rootContainer === doc) {
        parseAttributesInto(attrPart, doc.documentElement);
      }
      continue;
    }
    if (tagLower === "head") {
      if (!isClosing && doc && rootContainer === doc) {
        stack.push(doc.head);
      } else if (isClosing && stack[stack.length - 1] === doc?.head) {
        stack.pop();
      }
      continue;
    }
    if (tagLower === "body") {
      if (!isClosing && doc && rootContainer === doc) {
        stack.push(doc.body);
      } else if (isClosing && stack[stack.length - 1] === doc?.body) {
        stack.pop();
      }
      continue;
    }

    if (isClosing) {
      for (let s = stack.length - 1; s > 0; s -= 1) {
        if (stack[s].tagName === tagLower.toUpperCase()) {
          stack.length = s;
          break;
        }
      }
      continue;
    }

    const el = doc.createElement(tagLower);
    parseAttributesInto(attrPart, el);
    stack[stack.length - 1].appendChild(el);

    if (!explicitSelfClose && !VOID_TAGS.has(tagLower)) {
      stack.push(el);
    }
  }

  if (lastIndex < cleaned.length) {
    const tail = decodeEntities(cleaned.slice(lastIndex));
    if (tail) {
      stack[stack.length - 1].appendChild(new MockTextNode(tail, doc));
    }
  }
}

function parseHtmlFragment(markup, doc) {
  const frag = doc.createDocumentFragment();
  parseHtmlTokens(markup, frag, doc);
  return frag;
}

function parseHtmlDocument(htmlSource) {
  const doc = new MockDocument();
  parseHtmlTokens(htmlSource, doc, doc);
  return doc;
}

// ---------------------------------------------------------------------------
// Deterministic Sandbox Clock + WebAudio Stub + ESM-in-VM Loader
// ---------------------------------------------------------------------------

function createVirtualClock() {
  const baseEpoch = 1_730_000_000_000;
  let perfNow = 0;
  let nextTimerId = 1;
  let nextRafId = 1;
  const timers = new Map();
  const rafs = new Map();

  function setTimeoutFn(cb, delay = 0, ...args) {
    const id = nextTimerId++;
    const wait = Math.max(0, Number(delay) || 0);
    timers.set(id, { id, cb, args, dueAt: perfNow + wait, interval: null });
    return id;
  }

  function clearTimeoutFn(id) {
    timers.delete(id);
  }

  function setIntervalFn(cb, delay = 0, ...args) {
    const id = nextTimerId++;
    const period = Math.max(1, Number(delay) || 1);
    timers.set(id, { id, cb, args, dueAt: perfNow + period, interval: period });
    return id;
  }

  function clearIntervalFn(id) {
    timers.delete(id);
  }

  function requestAnimationFrameFn(cb) {
    const id = nextRafId++;
    rafs.set(id, cb);
    return id;
  }

  function cancelAnimationFrameFn(id) {
    rafs.delete(id);
  }

  class FakeDate extends Date {
    constructor(...args) {
      if (args.length === 0) {
        super(baseEpoch + perfNow);
      } else {
        super(...args);
      }
    }

    static now() {
      return baseEpoch + perfNow;
    }
  }

  async function stepFrame(dtMs = 16) {
    perfNow += dtMs;
    const queued = [...rafs.entries()];
    rafs.clear();
    for (const [, cb] of queued) {
      cb(perfNow);
    }
    await flushAsync(4);
  }

  async function advance(ms, { runFrames = false, frameStepMs = 16 } = {}) {
    const target = perfNow + Math.max(0, ms);
    while (perfNow < target) {
      let nextDue = target;
      for (const t of timers.values()) {
        if (t.dueAt < nextDue) nextDue = t.dueAt;
      }
      if (runFrames && rafs.size > 0) {
        nextDue = Math.min(nextDue, perfNow + frameStepMs);
      }
      const prevPerf = perfNow;
      perfNow = nextDue;

      if (runFrames && rafs.size > 0 && perfNow > prevPerf) {
        const queued = [...rafs.entries()];
        rafs.clear();
        for (const [, cb] of queued) {
          cb(perfNow);
        }
        await flushAsync(4);
      }

      const dueTimers = [...timers.values()]
        .filter((t) => t.dueAt <= perfNow)
        .sort((a, b) => a.dueAt - b.dueAt || a.id - b.id);

      for (const entry of dueTimers) {
        if (!timers.has(entry.id)) continue;
        if (entry.interval !== null) {
          entry.dueAt = perfNow + entry.interval;
        } else {
          timers.delete(entry.id);
        }
        entry.cb(...entry.args);
        await flushAsync(16);
      }

      if (perfNow === prevPerf && dueTimers.length === 0) {
        perfNow = target;
      }
    }
  }

  return {
    setTimeout: setTimeoutFn,
    clearTimeout: clearTimeoutFn,
    setInterval: setIntervalFn,
    clearInterval: clearIntervalFn,
    requestAnimationFrame: requestAnimationFrameFn,
    cancelAnimationFrame: cancelAnimationFrameFn,
    performance: { now: () => perfNow },
    Date: FakeDate,
    stepFrame,
    advance,
    activeRafCount: () => rafs.size,
    activeIntervalCount: () => [...timers.values()].filter((t) => t.interval !== null).length,
    activeTimeoutCount: () => [...timers.values()].filter((t) => t.interval === null).length,
  };
}

function createMockAudioContextClass(audioTracker) {
  function makeParam(initial = 0) {
    return {
      value: initial,
      setValueAtTime(v) {
        this.value = v;
      },
      linearRampToValueAtTime(v) {
        this.value = v;
      },
      exponentialRampToValueAtTime(v) {
        this.value = v;
      },
    };
  }

  return class MockAudioContext {
    constructor() {
      this.state = "running";
      this.currentTime = 0;
      this.sampleRate = 44100;
      this.destination = {};
      audioTracker.contexts.push(this);
    }

    async resume() {
      this.state = "running";
    }

    async suspend() {
      this.state = "suspended";
    }

    createGain() {
      return { gain: makeParam(1), connect() {} };
    }

    createOscillator() {
      return {
        type: "sine",
        frequency: makeParam(440),
        connect() {},
        start() {},
        stop() {},
      };
    }

    createBuffer(_channels, length) {
      const buf = new Float32Array(length);
      return { getChannelData: () => buf };
    }

    createBufferSource() {
      return { buffer: null, connect() {}, start() {} };
    }

    createBiquadFilter() {
      return { type: "bandpass", frequency: makeParam(1000), connect() {} };
    }
  };
}

function createWindowHarness({
  htmlPath,
  includeIcons = true,
  entryModule,
  invokeHandler,
  reducedMotion = false,
}) {
  const doc = parseHtmlDocument(readProjectFile(htmlPath));
  const clock = createVirtualClock();
  const audioTracker = { contexts: [] };
  const eventListeners = new Map();
  const invokeCalls = [];
  const storage = new Map();

  const localStorage = {
    getItem: (k) => (storage.has(String(k)) ? storage.get(String(k)) : null),
    setItem: (k, v) => storage.set(String(k), String(v)),
    removeItem: (k) => storage.delete(String(k)),
    clear: () => storage.clear(),
  };

  const tauriBridge = {
    core: {
      invoke: async (cmd, args = {}) => {
        invokeCalls.push({ cmd, args: structuredClone(args) });
        return invokeHandler(cmd, args);
      },
    },
    event: {
      listen: async (eventName, cb) => {
        if (!eventListeners.has(eventName)) {
          eventListeners.set(eventName, []);
        }
        eventListeners.get(eventName).push(cb);
        return () => {
          const arr = eventListeners.get(eventName) || [];
          const idx = arr.indexOf(cb);
          if (idx !== -1) arr.splice(idx, 1);
        };
      },
    },
    window: {
      getCurrentWindow: () => ({
        setTitle: async (t) => {
          doc.title = t;
        },
      }),
    },
  };

  const winTarget = new MockEventTarget();
  const sandbox = {
    console: {
      log() {},
      info() {},
      warn() {},
      error() {},
    },
    document: doc,
    Node: MockNode,
    Element: MockElement,
    HTMLElement: MockHTMLElement,
    HTMLInputElement: MockHTMLElement,
    HTMLSelectElement: MockHTMLElement,
    HTMLTextAreaElement: MockHTMLElement,
    HTMLButtonElement: MockHTMLElement,
    HTMLDivElement: MockHTMLElement,
    HTMLDialogElement: MockHTMLDialogElement,
    Event: MockEvent,
    CustomEvent: MockEvent,
    navigator: {
      language: "ru-RU",
      clipboard: { writeText: async () => {} },
    },
    location: {
      hash: "",
      search: "",
      href: "tauri://localhost/index.html",
      reload() {},
    },
    localStorage,
    setTimeout: clock.setTimeout,
    clearTimeout: clock.clearTimeout,
    setInterval: clock.setInterval,
    clearInterval: clock.clearInterval,
    requestAnimationFrame: clock.requestAnimationFrame,
    cancelAnimationFrame: clock.cancelAnimationFrame,
    performance: clock.performance,
    Date: clock.Date,
    structuredClone,
    URL,
    URLSearchParams,
    ResizeObserver: class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
    AudioContext: createMockAudioContextClass(audioTracker),
    getComputedStyle: (el) => ({
      display: el?.style?.display || "block",
      borderTopLeftRadius: "12px",
    }),
    matchMedia: (query) => ({
      matches: Boolean(reducedMotion && String(query).includes("reduce")),
      media: String(query),
      addEventListener() {},
      removeEventListener() {},
    }),
    addEventListener: winTarget.addEventListener.bind(winTarget),
    removeEventListener: winTarget.removeEventListener.bind(winTarget),
    dispatchEvent: winTarget.dispatchEvent.bind(winTarget),
    __TAURI__: tauriBridge,
  };
  sandbox.window = sandbox;
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;

  const context = vm.createContext(sandbox);

  if (includeIcons) {
    vm.runInContext(readProjectFile("src/icon-system.js"), context, {
      filename: "src/icon-system.js",
    });
  }

  const loadedModules = new Map();
  function requireSrcModule(specifier) {
    const cleanName = specifier.replace(/^\.\//, "");
    if (loadedModules.has(cleanName)) {
      return loadedModules.get(cleanName).exports;
    }
    const moduleRecord = { exports: {} };
    loadedModules.set(cleanName, moduleRecord);
    const cjsCode = transpileSrcModule(cleanName);
    const wrapped = `(function(exports, require, module) {\n${cjsCode}\n})`;
    const fn = vm.runInContext(wrapped, context, { filename: `src/${cleanName}` });
    fn(moduleRecord.exports, requireSrcModule, moduleRecord);
    return moduleRecord.exports;
  }

  requireSrcModule(entryModule);

  async function emitTauriEvent(eventName, payload) {
    const handlers = [...(eventListeners.get(eventName) || [])];
    for (const handler of handlers) {
      await handler({ event: eventName, payload });
    }
    await flushAsync(10);
  }

  return {
    document: doc,
    window: sandbox,
    clock,
    audioTracker,
    invokeCalls,
    localStorage,
    emitTauriEvent,
  };
}

function makeBaseSettings(overrides = {}) {
  return {
    transcription_mode: "local",
    ui_language: "ru",
    api_provider: "gemini",
    custom_api_url: "https://api.deepinfra.com/v1/openai",
    custom_model_name: "openai/whisper-large-v3-turbo",
    model_name: "base",
    hotkey: "Alt+V",
    streaming_enabled: false,
    toggle_enabled: false,
    language: "ru",
    dictionary: "",
    cloud_fallback_enabled: true,
    autostart: false,
    automatic_update_checks: false,
    local_engine: "whisper",
    local_acceleration: "cpu",
    overlay_sounds: true,
    overlay_sound_theme: "zen",
    overlay_sound_volume: 0.8,
    copy_context_on_start: false,
    audio_input_device: "default",
    log_speech_text: false,
    overlay_show_timer: true,
    gaming_mode_enabled: false,
    gaming_mode_auto_detect: true,
    vram_standby_timeout_mins: 30,
    text_replacements: [],
    ai_edit_mode: "off",
    translate_default_target: "en",
    llm_model: "qwen3-1.7b",
    has_completed_onboarding: true,
    disabled_edit_commands: [],
    smart_context_enabled: true,
    smart_context_trailing_period: true,
    has_api_key_gemini: false,
    has_api_key_openai: false,
    has_api_key_groq: false,
    has_api_key_huggingface: false,
    has_api_key_custom: false,
    ...overrides,
  };
}

async function bootSettingsWindow({ initialSettings, downloadedModels = ["base"], onInvoke }) {
  let storedSettings = structuredClone(initialSettings);
  const harness = createWindowHarness({
    htmlPath: "src/index.html",
    includeIcons: true,
    entryModule: "main.js",
    invokeHandler: async (cmd, args) => {
      if (onInvoke) {
        const custom = await onInvoke(cmd, args, {
          getStoredSettings: () => structuredClone(storedSettings),
          setStoredSettings: (next) => {
            storedSettings = structuredClone(next);
          },
        });
        if (custom !== undefined) return custom;
      }
      switch (cmd) {
        case "get_settings":
          return structuredClone(storedSettings);
        case "set_settings":
          storedSettings = { ...storedSettings, ...structuredClone(args.settings) };
          return null;
        case "set_ui_language":
          storedSettings.ui_language = args.uiLanguage;
          return null;
        case "get_downloaded_models":
          return [...downloadedModels];
        case "get_audio_input_devices":
          return ["default", "USB Studio Mic"];
        case "check_gpu_downloaded":
          return false;
        case "get_engine_health":
          return {
            engine: storedSettings.local_engine || "whisper",
            running: true,
            starting: false,
            gaming_mode: false,
            provider: "cpu",
            port: 51789,
          };
        case "get_history":
          return [];
        case "log_frontend_event":
          return null;
        default:
          return null;
      }
    },
  });

  harness.document.dispatchEvent(new harness.window.Event("DOMContentLoaded", { bubbles: true }));
  await flushAsync(24);
  return harness;
}

// ---------------------------------------------------------------------------
// 5 Required Behavioral Tests (Real Production Modules + Real HTML Markup)
// ---------------------------------------------------------------------------

test("3D-game auto-detection event never flips manual #checkbox-gaming-mode or poisons autosave", async () => {
  let runtimeGamingActive = false;
  const harness = await bootSettingsWindow({
    initialSettings: makeBaseSettings({
      gaming_mode_enabled: false,
      gaming_mode_auto_detect: true,
      autostart: false,
    }),
    onInvoke: async (cmd) => {
      if (cmd === "get_gaming_mode_state") {
        return runtimeGamingActive;
      }
      if (cmd === "get_engine_health") {
        return {
          engine: "whisper",
          running: !runtimeGamingActive,
          starting: false,
          gaming_mode: runtimeGamingActive,
          provider: "cpu",
          port: runtimeGamingActive ? null : 51789,
        };
      }
      return undefined;
    },
  });

  const checkboxGaming = harness.document.getElementById("checkbox-gaming-mode");
  const checkboxAutostart = harness.document.getElementById("checkbox-autostart");
  const healthChip = harness.document.getElementById("engine-health-chip");
  assert.ok(checkboxGaming && checkboxAutostart && healthChip);
  assert.equal(checkboxGaming.checked, false);

  // Simulate fullscreen 3D game detected by Rust background monitor:
  // runtime gaming state turns true, while manual settings.gaming_mode_enabled remains false.
  runtimeGamingActive = true;
  await harness.emitTauriEvent("gaming-mode-changed", true);

  assert.equal(
    checkboxGaming.checked,
    false,
    "auto-detected 3D game must not flip manual Gaming Mode checkbox",
  );
  assert.match(
    healthChip.textContent,
    /игровой режим/i,
    "engine health chip reflects active gaming standby",
  );

  // User changes an unrelated setting while the 3D game is still running:
  // saved settings must preserve gaming_mode_enabled: false.
  checkboxAutostart.click();
  await harness.clock.advance(200);

  const saveCalls = harness.invokeCalls.filter((c) => c.cmd === "set_settings");
  assert.equal(saveCalls.length, 1);
  assert.equal(saveCalls[0].args.settings.autostart, true);
  assert.equal(
    saveCalls[0].args.settings.gaming_mode_enabled,
    false,
    "autosave during auto-detected game must not persist manual gaming_mode_enabled=true",
  );
});

test("tray-menu rolls back optimistic UI and snapshot when backend rejects with model_not_installed:", async () => {
  const stored = makeBaseSettings({
    transcription_mode: "cloud",
    language: "ru",
    ui_language: "ru",
    gaming_mode_enabled: false,
  });

  const harness = createWindowHarness({
    htmlPath: "src/tray-menu.html",
    includeIcons: true,
    entryModule: "tray-menu.js",
    invokeHandler: async (cmd, args) => {
      if (cmd === "get_settings") {
        return structuredClone(stored);
      }
      if (cmd === "get_gaming_mode_state") {
        return false;
      }
      if (cmd === "set_transcription_mode") {
        if (args.mode === "local") {
          throw "model_not_installed:parakeet-v3";
        }
        stored.transcription_mode = args.mode;
        return null;
      }
      if (cmd === "set_recognition_language") {
        stored.language = args.language;
        return null;
      }
      if (cmd === "set_settings") {
        if (args.settings?.transcription_mode === "local") {
          throw "model_not_installed:parakeet-v3";
        }
        Object.assign(stored, structuredClone(args.settings));
        return null;
      }
      return null;
    },
  });

  await flushAsync(16);

  const subCloud = harness.document.getElementById("sub-cloud");
  const subLocal = harness.document.getElementById("sub-local");
  const recogTrigger = harness.document.getElementById("item-recog-trigger");
  const viewRecog = harness.document.getElementById("view-recog");
  const recogStatusIcon = harness.document.getElementById("recog-status-icon");
  assert.ok(subCloud && subLocal && recogTrigger && viewRecog && recogStatusIcon);

  assert.equal(subCloud.classList.contains("is-selected"), true);
  assert.equal(subCloud.getAttribute("aria-checked"), "true");
  assert.equal(subLocal.classList.contains("is-selected"), false);

  // Navigate to Recognition submenu and click Local AI (whose model is uninstalled)
  recogTrigger.click();
  assert.equal(viewRecog.classList.contains("active"), true);

  subLocal.click();
  await flushAsync(16);

  // Optimistic UI and internal state must roll back cleanly to Cloud AI
  assert.equal(
    subCloud.classList.contains("is-selected"),
    true,
    "Cloud AI must remain selected after model_not_installed: rejection",
  );
  assert.equal(subCloud.getAttribute("aria-checked"), "true");
  assert.equal(
    subLocal.classList.contains("is-selected"),
    false,
    "Local AI must not stay optimistically selected after backend rejection",
  );
  assert.equal(subLocal.getAttribute("aria-checked"), "false");
  assert.equal(recogStatusIcon.getAttribute("title"), "Облачный ИИ");

  // Failed click must not auto-return to main view after the 180ms submenu timer
  await harness.clock.advance(250);
  assert.equal(viewRecog.classList.contains("active"), true);

  // Verify tray-menu invoked the narrow set_transcription_mode command
  const modeCalls = harness.invokeCalls.filter((c) => c.cmd === "set_transcription_mode");
  assert.equal(modeCalls.length, 1);
  assert.deepEqual(modeCalls[0].args, { mode: "local" });
});

test("browsing an uninstalled local engine/model does not break autosave of other settings", async () => {
  const harness = await bootSettingsWindow({
    initialSettings: makeBaseSettings({
      transcription_mode: "local",
      local_engine: "whisper",
      model_name: "base",
      dictionary: "",
    }),
    downloadedModels: ["base"], // parakeet-v3 is NOT installed
    onInvoke: async (cmd, args) => {
      if (cmd === "set_settings") {
        // Mirror Rust validate_installed_models: reject if uninstalled parakeet-v3 leaks into payload
        if (
          args.settings.local_engine === "parakeet" ||
          args.settings.model_name === "parakeet-v3"
        ) {
          throw "model_not_installed:parakeet-v3";
        }
      }
      return undefined;
    },
  });

  const selectLocalEngine = harness.document.getElementById("select-local-engine");
  const groupParakeet = harness.document.getElementById("group-parakeet-models");
  const textareaDictionary = harness.document.getElementById("textarea-dictionary");
  const toastEl = harness.document.getElementById("app-toast");
  assert.ok(selectLocalEngine && groupParakeet && textareaDictionary && toastEl);

  // User switches engine dropdown to NVIDIA Parakeet to inspect / consider downloading it
  selectLocalEngine.value = "parakeet";
  selectLocalEngine.dispatchEvent(new harness.window.Event("change", { bubbles: true }));
  await harness.clock.advance(800);

  assert.equal(groupParakeet.style.display, "block", "Parakeet model card is shown for browsing");
  assert.equal(
    harness.invokeCalls.filter((c) => c.cmd === "set_settings").length,
    0,
    "browsing an uninstalled engine alone must not trigger a failing set_settings call",
  );

  // While still viewing uninstalled Parakeet, user updates their vocabulary on another tab
  textareaDictionary.value = "Aura, Parakeet, Tauri";
  textareaDictionary.dispatchEvent(new harness.window.Event("input", { bubbles: true }));
  await harness.clock.advance(750);

  const saveCalls = harness.invokeCalls.filter((c) => c.cmd === "set_settings");
  assert.equal(saveCalls.length, 1, "dictionary edit must autosave cleanly");
  assert.equal(saveCalls[0].args.settings.dictionary, "Aura, Parakeet, Tauri");
  assert.equal(
    saveCalls[0].args.settings.local_engine,
    "whisper",
    "committedLocalEngine stays on installed whisper engine",
  );
  assert.equal(
    saveCalls[0].args.settings.model_name,
    "base",
    "committedModelName stays on installed base model",
  );
  assert.equal(toastEl.classList.contains("toast-error"), false);
});

test("saved DPAPI provider key is protected on empty debounce and explicitly cleared across Speech/Text tabs", async () => {
  const harness = await bootSettingsWindow({
    initialSettings: makeBaseSettings({
      transcription_mode: "cloud",
      api_provider: "gemini",
      has_api_key_gemini: true,
    }),
  });

  const apiKeyInput = harness.document.getElementById("input-api-key");
  const apiKeyInputEdit = harness.document.getElementById("input-edit-api-key");
  const clearKeyBtn = harness.document.getElementById("btn-clear-key");
  const clearKeyEditBtn = harness.document.getElementById("btn-clear-key-edit");
  const textareaDictionary = harness.document.getElementById("textarea-dictionary");
  assert.ok(apiKeyInput && apiKeyInputEdit && clearKeyBtn && clearKeyEditBtn && textareaDictionary);

  const speechWrapper = apiKeyInput.closest(".api-key-input-wrapper");
  const editWrapper = apiKeyInputEdit.closest(".api-key-input-wrapper");
  assert.ok(speechWrapper && editWrapper);

  // 1. Initial saved-key state is reflected synchronously on both Speech and Text tabs
  assert.equal(apiKeyInput.value, "");
  assert.equal(apiKeyInputEdit.value, "");
  assert.match(apiKeyInput.placeholder, /••••••••/);
  assert.match(apiKeyInputEdit.placeholder, /••••••••/);
  assert.equal(clearKeyBtn.hidden, false);
  assert.equal(clearKeyEditBtn.hidden, false);
  assert.equal(speechWrapper.classList.contains("has-saved-key"), true);
  assert.equal(editWrapper.classList.contains("has-saved-key"), true);

  // 2. Typing a draft and clearing back to whitespace before 700ms debounce NEVER wipes DPAPI key
  apiKeyInput.value = "temp-draft";
  apiKeyInput.dispatchEvent(new harness.window.Event("input", { bubbles: true }));
  await harness.clock.advance(200);

  apiKeyInput.value = "   ";
  apiKeyInput.dispatchEvent(new harness.window.Event("input", { bubbles: true }));

  textareaDictionary.value = "Rust";
  textareaDictionary.dispatchEvent(new harness.window.Event("input", { bubbles: true }));
  await harness.clock.advance(750);

  assert.equal(harness.invokeCalls.filter((c) => c.cmd === "set_settings").length, 1);
  assert.equal(
    harness.invokeCalls.filter((c) => c.cmd === "set_provider_key").length,
    0,
    "empty API key input must never invoke set_provider_key(provider, '') on debounce",
  );
  assert.equal(clearKeyBtn.hidden, false);
  assert.equal(clearKeyEditBtn.hidden, false);
  assert.equal(speechWrapper.classList.contains("has-saved-key"), true);
  assert.equal(editWrapper.classList.contains("has-saved-key"), true);

  // 3. Explicit click on #btn-clear-key (Speech tab) clears key and syncs both Speech & Text tabs
  clearKeyBtn.click();
  await flushAsync(16);

  let setKeyCalls = harness.invokeCalls.filter((c) => c.cmd === "set_provider_key");
  assert.equal(setKeyCalls.length, 1);
  assert.deepEqual(setKeyCalls[0].args, { provider: "gemini", key: "" });
  assert.equal(clearKeyBtn.hidden, true, "#btn-clear-key must hide once key is cleared");
  assert.equal(clearKeyEditBtn.hidden, true, "#btn-clear-key-edit must hide in sync");
  assert.equal(speechWrapper.classList.contains("has-saved-key"), false);
  assert.equal(editWrapper.classList.contains("has-saved-key"), false);
  assert.doesNotMatch(apiKeyInput.placeholder, /••••••••/);
  assert.doesNotMatch(apiKeyInputEdit.placeholder, /••••••••/);
  assert.equal(harness.document.activeElement, apiKeyInput);

  // 4. Saving a new non-empty key re-enables .has-saved-key and unhides both clear buttons
  apiKeyInput.value = "AIzaSyReplacementKey999";
  apiKeyInput.dispatchEvent(new harness.window.Event("input", { bubbles: true }));
  await harness.clock.advance(750);

  setKeyCalls = harness.invokeCalls.filter((c) => c.cmd === "set_provider_key");
  assert.equal(setKeyCalls.length, 2);
  assert.deepEqual(setKeyCalls[1].args, {
    provider: "gemini",
    key: "AIzaSyReplacementKey999",
  });
  assert.equal(clearKeyBtn.hidden, false);
  assert.equal(clearKeyEditBtn.hidden, false);
  assert.equal(speechWrapper.classList.contains("has-saved-key"), true);
  assert.equal(editWrapper.classList.contains("has-saved-key"), true);
  assert.match(apiKeyInput.placeholder, /••••••••/);
  assert.match(apiKeyInputEdit.placeholder, /••••••••/);

  // 5. Explicit click on #btn-clear-key-edit (Text tab) clears shared provider key and syncs both tabs
  clearKeyEditBtn.click();
  await flushAsync(16);

  setKeyCalls = harness.invokeCalls.filter((c) => c.cmd === "set_provider_key");
  assert.equal(setKeyCalls.length, 3);
  assert.deepEqual(setKeyCalls[2].args, { provider: "gemini", key: "" });
  assert.equal(clearKeyBtn.hidden, true);
  assert.equal(clearKeyEditBtn.hidden, true);
  assert.equal(speechWrapper.classList.contains("has-saved-key"), false);
  assert.equal(editWrapper.classList.contains("has-saved-key"), false);
  assert.doesNotMatch(apiKeyInput.placeholder, /••••••••/);
  assert.doesNotMatch(apiKeyInputEdit.placeholder, /••••••••/);
  assert.equal(harness.document.activeElement, apiKeyInputEdit);
});

test("hidden overlay.js runs zero rAF or timer callbacks at idle and stops both after hiding", async () => {
  const harness = createWindowHarness({
    htmlPath: "src/overlay.html",
    includeIcons: false,
    entryModule: "overlay.js",
    reducedMotion: false,
    invokeHandler: async () => null,
  });

  await flushAsync(8);

  const pill = harness.document.querySelector(".overlay-pill");
  const statusText = harness.document.getElementById("overlay-status-text");
  assert.ok(pill && statusText);

  // 1. Hidden at startup: 0 rAF loops, 0 recording intervals
  assert.equal(pill.classList.contains("visible"), false);
  assert.equal(
    harness.clock.activeRafCount(),
    0,
    "hidden overlay must not schedule requestAnimationFrame at startup",
  );
  assert.equal(
    harness.clock.activeIntervalCount(),
    0,
    "hidden overlay must not run 500ms recording setInterval at startup",
  );

  // 2. Start recording: both rAF loop and 500ms timer activate
  await harness.emitTauriEvent("recording-state", "recording");
  assert.equal(pill.classList.contains("visible"), true);
  assert.equal(harness.clock.activeRafCount(), 1);
  assert.equal(harness.clock.activeIntervalCount(), 1);

  await harness.clock.advance(3500, { runFrames: true, frameStepMs: 50 });
  assert.equal(statusText.textContent, "0:03");
  assert.equal(harness.clock.activeRafCount(), 1);
  assert.equal(harness.clock.activeIntervalCount(), 1);

  // 3. Request hide on completion: timer stops immediately; after morph + fade, rAF stops completely
  await harness.emitTauriEvent("hide-overlay-requested", { status: "success" });
  assert.equal(
    harness.clock.activeIntervalCount(),
    0,
    "recording timer must stop immediately on hide-overlay-requested",
  );

  // Advance past the 520ms square morph delay so pill.classList.remove('visible') runs
  await harness.clock.advance(540, { runFrames: true, frameStepMs: 20 });
  assert.equal(pill.classList.contains("visible"), false);

  // Complete opacity fade transition
  const fadeEvent = new harness.window.Event("transitionend", {
    bubbles: true,
    propertyName: "opacity",
  });
  pill.dispatchEvent(fadeEvent);
  await flushAsync(8);

  assert.equal(
    harness.invokeCalls.filter((c) => c.cmd === "hide_overlay_window").length,
    1,
    "overlay must invoke hide_overlay_window once fade finishes",
  );
  assert.equal(
    harness.clock.activeRafCount(),
    0,
    "hidden overlay must cancel requestAnimationFrame once hidden",
  );
  assert.equal(
    harness.clock.activeIntervalCount(),
    0,
    "hidden overlay must keep 500ms interval stopped",
  );

  // Advance past the 2200ms idle WebAudio suspend timeout: AudioContext suspends and 0 timers remain
  await harness.clock.advance(2300, { runFrames: true, frameStepMs: 50 });
  assert.equal(harness.clock.activeRafCount(), 0);
  assert.equal(harness.clock.activeIntervalCount(), 0);
  assert.equal(harness.clock.activeTimeoutCount(), 0);
  if (harness.audioTracker.contexts.length > 0) {
    assert.equal(harness.audioTracker.contexts[0].state, "suspended");
  }
});
