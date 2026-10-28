// EGrab 规则脚本离线回归 —— 最小 DOM 实现
//
// fixtures/*.dom.json.gz 是真实商品页（浏览器渲染完成后另存的 MHTML）解析出的节点树，
// 本文件把它还原成规则脚本用得到的那部分 DOM API，在 Node 里直接执行 src-tauri/rules/*.js。
// 只实现规则脚本实际用到的选择器语法：tag / #id / .class / [a] / [a="v"] / [a*="v"] / [a^="v"]，
// 后代组合（空格）与逗号分组。

import fs from 'node:fs';
import zlib from 'node:zlib';

function parseCompound(s) {
  const c = { tag: null, id: null, classes: [], attrs: [] };
  const re = /^([a-zA-Z][\w-]*)|#([\w-]+)|\.([\w-]+)|\[([\w-]+)(?:([*^]?)="([^"]*)")?\]/g;
  let m;
  while ((m = re.exec(s.trim()))) {
    if (m[1]) c.tag = m[1].toLowerCase();
    else if (m[2]) c.id = m[2];
    else if (m[3]) c.classes.push(m[3]);
    else if (m[4]) c.attrs.push({ name: m[4], op: m[6] === undefined ? 'has' : (m[5] || '='), val: m[6] });
  }
  return c;
}

const selectorCache = new Map();
function parseSelector(sel) {
  if (!selectorCache.has(sel)) {
    selectorCache.set(sel, sel.split(',').map((g) => g.trim().split(/\s+/).map(parseCompound)));
  }
  return selectorCache.get(sel);
}

function matchCompound(el, c) {
  if (c.tag && el.tagName !== c.tag) return false;
  if (c.id && el.id !== c.id) return false;
  for (const cl of c.classes) if (!el._classSet.has(cl)) return false;
  for (const a of c.attrs) {
    const v = el.getAttribute(a.name);
    if (v === null) return false;
    if (a.op === '=' && v !== a.val) return false;
    if (a.op === '*' && v.indexOf(a.val) < 0) return false;
    if (a.op === '^' && v.indexOf(a.val) !== 0) return false;
  }
  return true;
}

function matchChain(el, chain, i) {
  if (!matchCompound(el, chain[i])) return false;
  if (i === 0) return true;
  for (let p = el.parentElement; p; p = p.parentElement) if (matchChain(p, chain, i - 1)) return true;
  return false;
}

function matches(el, groups) {
  return groups.some((g) => matchChain(el, g, g.length - 1));
}

class Element {
  constructor(json, parent) {
    this.tagName = json.t.toLowerCase();
    this._attrs = Object.assign({}, json.a || {});
    this._ownText = json.x || '';
    this.parentElement = parent;
    this.children = [];
    this._nodes = []; // 子元素与文本按原顺序混排，用于 textContent
    this.id = this._attrs.id || '';
    this.className = this._attrs.class || '';
    this._classSet = new Set(this.className.split(/\s+/).filter(Boolean));
    this.naturalWidth = 0;
    this.naturalHeight = 0;
    this.value = this._attrs.value;
    this.style = {};
    for (const child of json.c || []) {
      if (child.t === '#text') {
        this._nodes.push(child.x || '');
      } else {
        const el = new Element(child, this);
        this.children.push(el);
        this._nodes.push(el);
      }
    }
  }

  get classList() {
    const set = this._classSet;
    return { contains: (c) => set.has(c) };
  }

  getAttribute(name) {
    return Object.prototype.hasOwnProperty.call(this._attrs, name) ? this._attrs[name] : null;
  }

  setAttribute(name, value) {
    this._attrs[name] = String(value);
  }

  get textContent() {
    return this._ownText + this._nodes.map((n) => (typeof n === 'string' ? n : n.textContent)).join('');
  }

  getBoundingClientRect() {
    return { top: 0, left: 0, width: 0, height: 0 };
  }

  scrollIntoView() {}

  *descendants() {
    for (const c of this.children) {
      yield c;
      yield* c.descendants();
    }
  }

  querySelectorAll(sel) {
    const groups = parseSelector(sel);
    const out = [];
    for (const el of this.descendants()) if (matches(el, groups)) out.push(el);
    return out;
  }

  querySelector(sel) {
    const groups = parseSelector(sel);
    for (const el of this.descendants()) if (matches(el, groups)) return el;
    return null;
  }

  closest(sel) {
    const groups = parseSelector(sel);
    for (let el = this; el; el = el.parentElement) if (matches(el, groups)) return el;
    return null;
  }

  contains(other) {
    for (let el = other; el; el = el.parentElement) if (el === this) return true;
    return false;
  }

  remove() {
    const p = this.parentElement;
    if (!p) return;
    p.children = p.children.filter((c) => c !== this);
    p._nodes = p._nodes.filter((c) => c !== this);
    this.parentElement = null;
  }

  appendText(text) {
    this._nodes.push(text);
  }
}

// 载入 fixture，返回 { document, window }。每次调用都是一份全新的页面。
export function loadPage(fixturePath, url) {
  const raw = fs.readFileSync(fixturePath);
  const json = JSON.parse((fixturePath.endsWith('.gz') ? zlib.gunzipSync(raw) : raw).toString('utf8'));
  const root = new Element(json, null);
  const title = root.querySelector('title');
  const scrolls = [];
  const document = {
    documentElement: root,
    body: root.querySelector('body'),
    title: title ? title.textContent : '',
    visibilityState: 'visible',
    hasFocus: () => true,
    querySelector: (s) => root.querySelector(s),
    querySelectorAll: (s) => root.querySelectorAll(s),
    getElementById: (id) => {
      for (const el of root.descendants()) if (el.id === id) return el;
      return null;
    }
  };
  const window = {
    location: { href: url || '' },
    innerHeight: 900,
    pageYOffset: 0,
    scrolls,
    scrollTo: (x, y) => { scrolls.push(y); window.pageYOffset = y; },
    scrollBy: (x, y) => { window.scrollTo(0, window.pageYOffset + y); },
    getComputedStyle: () => ({ backgroundImage: 'none' })
  };
  return { document, window };
}

// 在给定页面上执行一个规则脚本文件（脚本是一个立即执行的表达式）。
export function runScript(page, scriptPath) {
  const src = fs.readFileSync(scriptPath, 'utf8');
  return new Function('document', 'window', 'return (' + src + ');')(page.document, page.window);
}
