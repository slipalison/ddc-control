// The dev guard against text no translation produced (D-2026-09-27-tray-
// app-11). On the demo of a local dev server (D-2026-09-27-tray-app-6),
// every text `t()` answers is noted, and every text the page comes to show
// is checked once the task that wrote it ends: a text node or a readable
// attribute, whichever helper or property wrote it. One that has a letter,
// that `t()` never answered, and that is not under `translate="no"` (the
// monitor's own data, D-2026-09-27-tray-app-7) is a console error, and the
// Playwright suite fails on any: so every path a test walks checks the
// texts it shows. Checking when the task ends, rather than at each write,
// lets a node take its text first and its `translate="no"` right after,
// as the popup's data helpers do. Inside the app the guard is off: `t()`
// stays as it is and nothing watches the page.

/** What every report of the guard starts with. */
export const UNTRANSLATED = 'ddc-tray: untranslated text:';

/** Attributes a person reads or hears: the guard checks them as texts. */
export const GUARDED_ATTRIBUTES = Object.freeze([
  'aria-label',
  'aria-description',
  'aria-roledescription',
  'aria-valuetext',
  'aria-placeholder',
  'aria-braillelabel',
  'aria-brailleroledescription',
  'aria-keyshortcuts',
  'title',
  'alt',
  'placeholder',
  'label',
  'value',
]);

/** What the guard asks a `MutationObserver` for: every text, anywhere. */
export const OBSERVED = Object.freeze({
  childList: true,
  subtree: true,
  characterData: true,
  attributes: true,
  attributeFilter: GUARDED_ATTRIBUTES,
});

const LETTER = /\p{L}/u;
const TEXT_NODE = 3;

const OFF = Object.freeze({
  enabled: false,
  track: (tr) => tr,
  check: () => true,
  watch: () => () => {},
});

/**
 * The guard: on only when `enabled` (the demo of a local dev server);
 * `report` hears each text it finds. Off, `track` hands the translator
 * back untouched and nothing is ever checked or watched.
 * @param {{ enabled: boolean, report?: (message: string) => void }} options
 */
export function textGuard({ enabled, report = (message) => console.error(message) }) {
  if (!enabled) return OFF;
  const produced = new Set();
  const check = (text, element) => {
    if (!LETTER.test(text) || produced.has(text) || element?.translate === false) return true;
    report(`${UNTRANSLATED} ${JSON.stringify(text)} in ${where(element)}`);
    return false;
  };
  const checkAll = (entries) => {
    for (const { text, element } of entries) check(text, element);
  };
  return Object.freeze({
    enabled: true,
    /** `tr`, noting each text it answers; a key no locale has is no text. */
    track: (tr) => (key, params) => {
      const text = tr(key, params);
      if (text !== key) produced.add(text);
      return text;
    },
    /** Whether `text`, shown by `element`, is a translation or data; reports it when not. */
    check,
    /** Checks the texts under `root`, then each text the page gains; returns what stops it. */
    watch: (root, Observer) => {
      checkAll(textsIn(root));
      const observer = new Observer((records) => checkAll(writtenTexts(records)));
      observer.observe(root, OBSERVED);
      return () => observer.disconnect();
    },
  });
}

/**
 * The texts that mutation `records` left on the page, each as
 * `{ text, element }`: added text nodes, the text nodes and readable
 * attributes of added elements, changed text nodes and changed readable
 * attributes. What left the page again before the check is not shown.
 * @param {Iterable<MutationRecord>} records
 */
export function writtenTexts(records) {
  const found = [];
  for (const record of records) {
    if (record.type === 'childList') {
      for (const node of record.addedNodes) if (node.isConnected) found.push(...textsIn(node));
    } else if (record.type === 'characterData' && record.target.isConnected) {
      found.push({ text: record.target.data, element: record.target.parentElement });
    } else if (record.type === 'attributes' && record.target.isConnected) {
      found.push(...attributeTexts(record.target, [record.attributeName]));
    }
  }
  return found;
}

/** The texts `node` shows: its own if it is a text node, else its readable attributes and its children's. */
function textsIn(node) {
  if (node.nodeType === TEXT_NODE) return [{ text: node.data, element: node.parentElement }];
  const own = typeof node.getAttribute === 'function' ? attributeTexts(node, GUARDED_ATTRIBUTES) : [];
  return [...own, ...[...(node.childNodes ?? [])].flatMap(textsIn)];
}

function attributeTexts(element, names) {
  return names
    .filter((name) => GUARDED_ATTRIBUTES.includes(name))
    .map((name) => element.getAttribute(name))
    .filter((value) => value !== null)
    .map((text) => ({ text, element }));
}

function where(element) {
  if (!element) return '<detached>';
  const classes = [...(element.classList ?? [])].map((name) => `.${name}`).join('');
  return `<${element.localName}${element.id ? `#${element.id}` : ''}${classes}>`;
}
