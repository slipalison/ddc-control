import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';

import en from '../../src/i18n/en.js';
import ptBR from '../../src/i18n/pt-BR.js';
import { ICONS, createIcon } from '../../src/icons.js';

// Every text of the popup comes from a key (D-2026-09-26-tray-app-6), and
// under the strict CSP (D-2026-09-26-tray-app-7) nothing inline runs or
// styles it. These read the popup's sources as text: the page itself is
// checked in a browser by the Playwright suite.

const source = (path) => readFileSync(new URL(`../../src/${path}`, import.meta.url), 'utf8');

const html = source('index.html');
const app = source('app.js');
const viewModel = source('view-model.js');
const dropdown = source('dropdown.js');
const scripts = {
  'app.js': app,
  'view-model.js': viewModel,
  'icons.js': source('icons.js'),
  'dropdown.js': dropdown,
};

/** Attributes a person reads or hears: they may only come from keys. */
const READABLE_ATTRIBUTES = [
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
];

const locales = { en, 'pt-BR': ptBR };

function missingKeys(keys) {
  return Object.entries(locales).flatMap(([locale, texts]) =>
    keys.filter((key) => !Object.hasOwn(texts, key)).map((key) => `${locale}: ${key}`),
  );
}

const attributeValues = (name) => [...html.matchAll(new RegExp(`\\s${name}="([^"]*)"`, 'g'))].map(([, value]) => value);

const i18nKeys = attributeValues('data-i18n');
const i18nAttrPairs = attributeValues('data-i18n-attr').flatMap((value) =>
  value.split(';').map((pair) => pair.split(':').map((part) => part.trim())),
);

// Literal keys of `t('…')` calls; keys built at run time come from the
// view-model's DTO mapping and are checked by i18n-parity.test.mjs.
const literalKeys = (code) => [...code.matchAll(/\bt\(\s*(['"])([^'"]+)\1/g)].map(([, , key]) => key);

test('the page has no literal text: every text node is empty', () => {
  const text = html
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/<[^>]+>/g, '')
    .trim();

  assert.equal(text, '');
});

test('the page has no literal readable attribute', () => {
  const found = READABLE_ATTRIBUTES.filter((name) => new RegExp(`\\s${name}\\s*=`).test(html));

  assert.deepEqual(found, []);
});

test('the page is translated through data-i18n and data-i18n-attr keys', () => {
  assert.ok(i18nKeys.length >= 10, `only ${i18nKeys.length} data-i18n keys`);
  assert.ok(i18nAttrPairs.length >= 3, `only ${i18nAttrPairs.length} data-i18n-attr pairs`);
  assert.ok(i18nKeys.includes('app.title'), 'the title comes from a key');
});

test('every data-i18n and data-i18n-attr key exists in both locales', () => {
  const keys = [...i18nKeys, ...i18nAttrPairs.map(([, key]) => key)];

  assert.deepEqual(missingKeys(keys), []);
});

test('data-i18n-attr only sets readable attributes, each as attribute:key', () => {
  for (const pair of i18nAttrPairs) {
    assert.equal(pair.length, 2, `malformed pair ${pair.join(':')}`);
    assert.ok(READABLE_ATTRIBUTES.includes(pair[0]), `${pair[0]} is not a readable attribute`);
  }
});

test("every literal t('…') key of app.js and view-model.js exists in both locales", () => {
  const keys = [...new Set([...literalKeys(app), ...literalKeys(viewModel)])];

  assert.ok(literalKeys(app).length >= 15, `only ${literalKeys(app).length} t() calls in app.js`);
  assert.deepEqual(missingKeys(keys), []);
});

test('app.js never writes a literal string as text', () => {
  const literalText = [...app.matchAll(/\.textContent\s*=\s*(['"`])[^'"`]*[A-Za-z]/g)].map(([match]) => match);

  assert.deepEqual(literalText, []);
});

// The DOM the scripts build takes its texts from keys or from the monitor's
// data: a literal handed to a helper, a text property or a readable
// attribute would be the one text a translation never reaches.
test('app.js passes no literal text to element() or setText()', () => {
  const scans = new Map(domModules().map(([name, code]) => [name, literalDomText(code)]));

  assert.deepEqual(
    ['app.js', 'dropdown.js', 'icons.js'].filter((name) => !scans.has(name)),
    [],
    'modules that build the DOM',
  );
  const { sites } = scans.get('app.js');
  for (const [how, least] of [
    ['element()', 8],
    ['setText()', 4],
    ['.textContent =', 10],
    ['setAttribute()', 10],
  ]) {
    assert.ok((sites.get(how) ?? 0) >= least, `only ${sites.get(how) ?? 0} ${how} in app.js`);
  }
  assert.ok((scans.get('dropdown.js').sites.get('element()') ?? 0) >= 1, 'the dropdown element() was read');
  const found = [...scans].flatMap(([name, { found: literals }]) =>
    literals.map(({ line, how, literal }) => `${name}:${line} ${how} ${JSON.stringify(literal)}`),
  );
  assert.deepEqual(found, []);
});

test('the literal-text scan flags each way a text reaches the DOM, and nothing else', () => {
  const code = [
    "function element(tag, className = '', text = null) {}",
    'function setText(node, text) {}',
    "element('span', 'tag', 'Undeclared');",
    "element('span', 'tag', t('tag.dangerous'));",
    "element('span', 'tag', view.label);",
    "setText(ui.note, busy ? 'Saving' : null);",
    'setText(ui.note, `${count} left`);',
    "node.textContent = 'Hello';",
    "node.textContent = text ?? '';",
    'node.title = "Close";',
    "node.setAttribute('aria-label', 'Close');",
    "node.setAttribute('title', t('action.close'));",
    "node.setAttribute('aria-hidden', 'true');",
    "node.setAttribute('aria-controls', `${id}-list`);",
    "node.setAttribute('aria-live', 'Loud');",
    "Object.assign(node, { type: 'button', textContent: 'Go' });",
    "node.append(createIcon('check'), 'Done');",
    "// element('span', 'tag', 'In a comment');",
    "const quote = /'/; node.textContent = 'After a regex'; const again = /'/;",
    "const half = width / 2; node.ariaLabel = 'After a division';",
    "node.textContent += ' more';",
    "/* element('span', 'tag', 'In a block comment'); */",
  ].join('\n');

  const found = literalDomText(code).found.map(({ line, how, literal }) => `${line} ${how} ${JSON.stringify(literal)}`);

  assert.deepEqual(found, [
    '3 element() "Undeclared"',
    '6 setText() "Saving"',
    '7 setText() "${} left"',
    '8 .textContent = "Hello"',
    '10 .title = "Close"',
    '11 setAttribute() "Close"',
    '15 setAttribute() "Loud"',
    '16 Object.assign() "Go"',
    '17 append() "Done"',
    '19 .textContent = "After a regex"',
    '20 .ariaLabel = "After a division"',
    '21 .textContent += " more"',
  ]);
});

test('the only script is the app module, and nothing is inline', () => {
  const scriptTags = [...html.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)];

  assert.deepEqual(
    scriptTags.map(([tag, body]) => [tag.slice(0, tag.indexOf('>') + 1), body.trim()]),
    [['<script type="module" src="app.js">', '']],
  );
  assert.doesNotMatch(html, /<style\b/);
  assert.doesNotMatch(html, /\sstyle\s*=/);
  assert.doesNotMatch(html, /\son[a-z]+\s*=/);
  assert.doesNotMatch(html, /javascript:/i);
  assert.match(html, /<link rel="stylesheet" href="styles\.css">/);
});

test('scripts never parse markup, set a style attribute or evaluate code', () => {
  const forbidden = [
    /\.innerHTML\b/,
    /\.outerHTML\b/,
    /insertAdjacentHTML/,
    /document\.write/,
    /setAttribute\(\s*['"]style['"]/,
    /\.style\.cssText/,
    /\beval\(/,
    /new Function\(/,
    /\.setAttribute\(\s*['"]on/,
  ];
  for (const [name, code] of Object.entries(scripts)) {
    const found = forbidden.filter((pattern) => pattern.test(code)).map(String);
    assert.deepEqual(found, [], name);
  }
});

// A text a style generates reaches no key either, in any state, visited
// by the pseudo-locale spec or not.
test('no style writes a text: every content of styles.css is empty', () => {
  const css = source('styles.css').replace(/\/\*[\s\S]*?\*\//g, '');
  const contents = [...css.matchAll(/[{;]\s*content\s*:\s*([^;}]*)/g)].map(([, value]) => value.trim());

  assert.ok(contents.length >= 1, 'the content declarations were read');
  assert.deepEqual(
    contents.filter((value) => !['""', "''", 'none', 'normal'].includes(value)),
    [],
  );
});

test('a slider fill is set as a custom property, the one style the script touches', () => {
  const styleWrites = [...app.matchAll(/\.style\.(\w+)/g)].map(([, member]) => member);

  assert.deepEqual([...new Set(styleWrites)], ['setProperty']);
  assert.match(app, /style\.setProperty\('--fill'/);
});

test('the dropdown only sets its --dropdown-* custom properties, never a literal text', () => {
  const styleWrites = [...dropdown.matchAll(/\.style\.(\w+)\(([^,]*)/g)].map(([, member, name]) => [member, name]);

  assert.ok(styleWrites.length > 0, 'the dropdown places its list');
  assert.deepEqual(
    styleWrites.filter(([member, name]) => member !== 'setProperty' || name !== '`--dropdown-${name}`'),
    [],
  );
  assert.deepEqual([...dropdown.matchAll(/\.textContent\s*=\s*(['"`])/g)].map(([match]) => match), []);
  assert.deepEqual(literalKeys(dropdown), [], 'its texts come from the caller');
});

test('no script builds a native select element', () => {
  for (const [name, code] of Object.entries(scripts)) {
    assert.doesNotMatch(code, /createElement\(\s*['"]select['"]|HTMLSelectElement/, name);
  }
  assert.doesNotMatch(html, /<select\b/i);
});

test('every icon the page and the app ask for exists', () => {
  const quoted = (text) => [...text.matchAll(/'([\w-]+)'/g)].map(([, name]) => name);
  const [, controlIcons = ''] = app.match(/const CONTROL_ICONS = Object\.freeze\(\{([\s\S]*?)\}\)/) ?? [];
  const calls = [...app.matchAll(/\b(?:createIcon|tile)\(([^)]*)\)/g)].map(([, args]) => args);
  const asked = [...attributeValues('data-icon'), ...quoted(controlIcons), ...calls.flatMap(quoted)];

  assert.ok(quoted(controlIcons).length >= 5, 'CONTROL_ICONS was read');
  assert.deepEqual(
    asked.filter((name) => !Object.hasOwn(ICONS, name)),
    [],
  );
});

// An error's message may reach the page as the error state's detail line,
// which shows data: an unknown icon's is its name alone.
test('an unknown icon throws a RangeError whose message is its name alone', () => {
  assert.throws(() => createIcon('no-such-icon', null), { name: 'RangeError', message: 'no-such-icon' });
});

// ------------------------------------------------ literal text in the DOM
//
// A small tokenizer, enough of JavaScript for the sources of `src/` (not a
// parser): it drops comments and tells strings, templates and regexes
// apart, so a quote in a regex or a call in a comment never throws off the
// scan of the places a text reaches the page.

const LETTER = /\p{L}/u;
const OPENERS = new Set(['(', '[', '{']);
const CLOSERS = new Set([')', ']', '}']);
/** Tokens of more than one character, longest first. */
const PUNCTUATORS = '... === !== ??= ||= &&= => == != <= >= && || ?? ?. += -='.split(' ');
/** Assignments that can put a text into a property. */
const ASSIGNMENTS = new Set(['=', '+=', '??=', '||=', '&&=']);
/** Words after which a `/` opens a regex rather than divides. */
const REGEX_AFTER_WORDS = new Set('return typeof case in of new delete void throw else do yield await'.split(' '));

/** DOM properties whose value a person reads or hears. */
const TEXT_PROPERTIES = new Set([
  'textContent',
  'innerText',
  'outerText',
  'nodeValue',
  'title',
  'alt',
  'placeholder',
  'ariaLabel',
  'ariaDescription',
  'ariaRoleDescription',
  'ariaValueText',
  'ariaPlaceholder',
  'ariaKeyShortcuts',
]);
/** ARIA attributes that hold element ids, never read out. */
const ARIA_IDREFS = new Set([
  'aria-activedescendant',
  'aria-controls',
  'aria-describedby',
  'aria-details',
  'aria-errormessage',
  'aria-flowto',
  'aria-labelledby',
  'aria-owns',
]);
/** Every token value of ARIA's enumerated and true/false attributes. */
const ARIA_TOKENS = new Set(
  (
    'true false mixed undefined off polite assertive horizontal vertical none inline list both ' +
    'ascending descending other menu listbox tree grid dialog page step location date time ' +
    'grammar spelling additions removals text all'
  ).split(' '),
);
/** Methods that add their arguments as children: a string one is a text node. */
const CHILD_METHODS = new Set(['append', 'prepend', 'replaceChildren', 'before', 'after', 'replaceWith']);
const STATEMENT_END = new Set([';', ',']);

/** The scripts of `src/` that build the DOM, by path: `[path, source]`. */
function domModules() {
  const root = new URL('../../src/', import.meta.url);
  return readdirSync(root, { recursive: true })
    .filter((path) => path.endsWith('.js'))
    .sort()
    .map((path) => [path, readFileSync(new URL(path, root), 'utf8')])
    .filter(([, code]) => buildsDom(tokenize(code).tokens));
}

function buildsDom(tokens) {
  return tokens.some(
    ({ kind, text }) =>
      kind === 'word' && ['document', 'createElement', 'createElementNS', 'textContent', 'setAttribute'].includes(text),
  );
}

/**
 * The places `code` puts literal text into the page: a call of one of its
 * own helpers with a `text` parameter (`element()`, `setText()`…), a text
 * property, a readable attribute, `Object.assign` or a child added as a
 * string. `found` lists them as `{ line, how, literal }`; `sites` counts
 * the places read, by `how`. Only `t(…)` may hold a literal: its key.
 */
function literalDomText(code) {
  const { tokens } = tokenize(code);
  const helpers = textHelpers(tokens);
  const found = [];
  const sites = new Map();
  const read = (how, literals) => {
    sites.set(how, (sites.get(how) ?? 0) + 1);
    for (const literal of literals) found.push({ line: lineOf(code, literal.at), how, literal: literal.text });
  };
  tokens.forEach((token, index) => {
    if (token.kind !== 'word') return;
    const before = tokens[index - 1]?.text;
    const after = tokens[index + 1]?.text;
    const member = before === '.' || before === '?.';
    const args = () => callArguments(tokens, index + 1);
    if (after === '(' && !member && before !== 'function' && helpers.has(token.text)) {
      read(`${token.text}()`, literalTexts(args()[helpers.get(token.text)] ?? []));
    } else if (ASSIGNMENTS.has(after) && member && TEXT_PROPERTIES.has(token.text)) {
      read(`.${token.text} ${after}`, literalTexts(expressionFrom(tokens, index + 2)));
    } else if (after === '(' && member && token.text === 'setAttribute') {
      read('setAttribute()', attributeLiterals(args()));
    } else if (after === '(' && member && token.text === 'assign' && tokens[index - 2]?.text === 'Object') {
      read('Object.assign()', args().slice(1).flatMap(propertyLiterals));
    } else if (after === '(' && member && CHILD_METHODS.has(token.text)) {
      read(`${token.text}()`, args().flatMap((arg) => literalTexts(arg, { skipCalls: true })));
    } else if (after === '(' && member && token.text === 'createTextNode') {
      read('createTextNode()', literalTexts(args()[0] ?? []));
    } else if (after === '(' && member && token.text === 'insertAdjacentText') {
      read('insertAdjacentText()', literalTexts(args()[1] ?? []));
    }
  });
  return { found, sites };
}

/** The module's functions with a `text` parameter, and its position. */
function textHelpers(tokens) {
  const helpers = new Map();
  tokens.forEach((token, index) => {
    if (token.text !== 'function' || tokens[index + 1]?.kind !== 'word' || tokens[index + 2]?.text !== '(') return;
    const at = callArguments(tokens, index + 2).findIndex((param) => param[0]?.text === 'text');
    if (at >= 0) helpers.set(tokens[index + 1].text, at);
  });
  return helpers;
}

function attributeLiterals([name = [], value = []]) {
  const attribute = name.length === 1 && name[0].kind === 'string' ? name[0].text : null;
  if (attribute === null || READABLE_ATTRIBUTES.includes(attribute)) return literalTexts(value);
  if (!attribute.startsWith('aria-') || ARIA_IDREFS.has(attribute)) return [];
  return literalTexts(value).filter(({ text }) => !text.split(/\s+/).every((word) => ARIA_TOKENS.has(word)));
}

/** The literals of the text properties of an object literal argument. */
function propertyLiterals(arg) {
  if (arg[0]?.text !== '{') return [];
  return callArguments(arg, 0)
    .filter(([key, colon]) => colon?.text === ':' && TEXT_PROPERTIES.has(key.text))
    .flatMap((entry) => literalTexts(entry.slice(2)));
}

/**
 * The string and template literals in `tokens` that hold a letter, but
 * those in `t(…)`; with `skipCalls`, those in any call.
 */
function literalTexts(tokens, { skipCalls = false } = {}) {
  const found = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const called = token.kind === 'word' && tokens[index + 1]?.text === '(';
    const member = ['.', '?.'].includes(tokens[index - 1]?.text);
    if (called && (skipCalls || (token.text === 't' && !member))) {
      index = closing(tokens, index + 1);
    } else if (token.kind === 'string' || token.kind === 'template') {
      if (LETTER.test(token.text)) found.push(token);
      for (const expression of token.expressions ?? []) found.push(...literalTexts(expression, { skipCalls }));
    }
  }
  return found;
}

/** The arguments of the call (or entries of the literal) opened at `open`, each a token list. */
function callArguments(tokens, open) {
  const end = closing(tokens, open);
  const args = [];
  let current = [];
  let depth = 0;
  for (let index = open + 1; index < end; index += 1) {
    const token = tokens[index];
    if (token.kind === 'punct' && OPENERS.has(token.text)) depth += 1;
    if (token.kind === 'punct' && CLOSERS.has(token.text)) depth -= 1;
    if (depth === 0 && token.text === ',' && token.kind === 'punct') {
      args.push(current);
      current = [];
    } else {
      current.push(token);
    }
  }
  if (current.length > 0) args.push(current);
  return args;
}

/** The expression from `start` to the end of its statement, argument or bracket. */
function expressionFrom(tokens, start) {
  const expression = [];
  let depth = 0;
  for (const token of tokens.slice(start)) {
    if (token.kind === 'punct' && depth === 0 && (STATEMENT_END.has(token.text) || CLOSERS.has(token.text))) break;
    if (token.kind === 'punct' && OPENERS.has(token.text)) depth += 1;
    if (token.kind === 'punct' && CLOSERS.has(token.text)) depth -= 1;
    expression.push(token);
  }
  return expression;
}

/** The index of the token that closes the bracket opened at `open`. */
function closing(tokens, open) {
  let depth = 0;
  for (let index = open; index < tokens.length; index += 1) {
    const { kind, text } = tokens[index];
    if (kind !== 'punct') continue;
    if (OPENERS.has(text)) depth += 1;
    if (CLOSERS.has(text)) depth -= 1;
    if (depth === 0) return index;
  }
  return tokens.length;
}

function lineOf(code, at) {
  return code.slice(0, at).split('\n').length;
}

/**
 * The tokens of `code` from `start`, comments left out: `string`,
 * `template` (its static text, `${}` where each expression was, and the
 * tokens of each), `regex`, `word`, `number` and `punct`, each with its
 * offset `at`. With `closer`, stops after the `}` that closes a `${`.
 */
function tokenize(code, start = 0, closer = false) {
  const tokens = [];
  let index = start;
  let depth = 0;
  const push = (kind, end, extra = {}) => {
    tokens.push({ kind, text: code.slice(index, end), at: index, ...extra });
    index = end;
  };
  while (index < code.length) {
    const char = code[index];
    const rest = code.slice(index, index + 3);
    if (/\s/.test(char)) index += 1;
    else if (rest.startsWith('//')) index = endOfLine(code, index);
    else if (rest.startsWith('/*')) index = endOfComment(code, index);
    else if (char === "'" || char === '"') {
      const end = stringEnd(code, index);
      tokens.push({ kind: 'string', text: code.slice(index + 1, end - 1), at: index });
      index = end;
    } else if (char === '`') {
      const template = readTemplate(code, index);
      tokens.push(template);
      index = template.end;
    } else if (char === '/' && regexAllowed(tokens.at(-1))) push('regex', regexEnd(code, index));
    else if (/[A-Za-z_$]/.test(char)) push('word', matchEnd(code, index, /[\w$]*/y));
    else if (/\d/.test(char)) push('number', matchEnd(code, index, /[\w.]*/y));
    else if (closer && char === '}' && depth === 0) return { tokens, end: index + 1 };
    else {
      const text = PUNCTUATORS.find((candidate) => code.startsWith(candidate, index)) ?? char;
      if (OPENERS.has(text)) depth += 1;
      if (CLOSERS.has(text)) depth -= 1;
      push('punct', index + text.length);
    }
  }
  return { tokens, end: index };
}

function readTemplate(code, start) {
  const parts = [];
  const expressions = [];
  let text = '';
  let index = start + 1;
  while (index < code.length && code[index] !== '`') {
    if (code[index] === '\\') {
      text += code.slice(index, index + 2);
      index += 2;
    } else if (code.startsWith('${', index)) {
      parts.push(text);
      text = '';
      const inner = tokenize(code, index + 2, true);
      expressions.push(inner.tokens);
      index = inner.end;
    } else {
      text += code[index];
      index += 1;
    }
  }
  parts.push(text);
  return { kind: 'template', text: parts.join('${}'), expressions, at: start, end: index + 1 };
}

function stringEnd(code, start) {
  let index = start + 1;
  while (index < code.length && code[index] !== code[start]) index += code[index] === '\\' ? 2 : 1;
  return index + 1;
}

function regexAllowed(previous) {
  if (!previous) return true;
  if (previous.kind === 'word') return REGEX_AFTER_WORDS.has(previous.text);
  return previous.kind === 'punct' && !CLOSERS.has(previous.text);
}

function regexEnd(code, start) {
  let index = start + 1;
  let inClass = false;
  while (index < code.length && (inClass || code[index] !== '/')) {
    if (code[index] === '\n') throw new Error(`unterminated regex at offset ${start}`);
    if (code[index] === '[') inClass = true;
    if (code[index] === ']') inClass = false;
    index += code[index] === '\\' ? 2 : 1;
  }
  return matchEnd(code, index + 1, /[a-z]*/y);
}

function matchEnd(code, start, sticky) {
  sticky.lastIndex = start;
  sticky.test(code);
  return sticky.lastIndex;
}

function endOfComment(code, start) {
  const end = code.indexOf('*/', start + 2);
  return end < 0 ? code.length : end + 2;
}

function endOfLine(code, start) {
  const end = code.indexOf('\n', start);
  return end < 0 ? code.length : end;
}

// ---------------------------------------- natural language in the scripts
//
// D-2026-09-27-tray-app-9. The scan above follows a text to the places it
// enters the page, and the pseudo-locale spec only sees the states it
// renders: a sentence parked in a const, picked by a ternary or handed
// through a sink neither knows gets past both. So no string or template
// literal of the popup's scripts may hold a phrase — two or more words of
// two or more letters, apart by whitespace — whatever it is for, but in
// the locale files (`i18n/en.js`, `i18n/pt-BR.js`: the texts themselves)
// and in `demo-data.js` (the demo's monitors and backend, written out as
// the core and Tauri answer). `i18n/index.js` is code, and is read. A
// template is read with `${}` where each expression was, each literal
// inside an expression on its own; literals joined with `+` are read as
// one, a plain operand between two of them as `${}`; escapes are read as
// the characters they stand for.

/** Paths under `src/` whose literals are texts or data by design, exactly. */
const LANGUAGE_FILES = new Set(['i18n/en.js', 'i18n/pt-BR.js', 'demo-data.js']);

const isLanguageFile = (path) => LANGUAGE_FILES.has(path);

/**
 * Literals with two words that are not language, by file, each with why.
 * An entry names one literal exactly (as read: `${}` for an expression),
 * and must still be in its file.
 */
const NOT_LANGUAGE = [
  { file: 'app.js', literal: 'slider has-icon', why: 'the class names of a slider with an icon (className)' },
  { file: 'app.js', literal: 'feature-row is-silent', why: 'the class names of a feature with no value (className)' },
  { file: 'icons.js', literal: 'icon icon-${}', why: "the class names of an icon's svg (its class attribute)" },
];

const WORD = /\p{L}{2,}/u;
const ESCAPED = /\\(?:u\{([\da-fA-F]+)\}|u([\da-fA-F]{4})|x([\da-fA-F]{2})|(\r\n|[\s\S]))/g;
const SINGLE_ESCAPES = { n: '\n', t: '\t', r: '\r', v: '\v', f: '\f', b: '\b', 0: '\0' };

/** Whether `text` has two or more whitespace-separated words of two or more letters. */
function isPhrase(text) {
  return text.split(/\s+/u).filter((chunk) => WORD.test(chunk)).length >= 2;
}

/** A literal's source text as the characters it stands for. */
function cooked(raw) {
  return raw.replace(ESCAPED, (escape, braced, unicode, byte, other) => {
    const code = braced ?? unicode ?? byte;
    if (code !== undefined) return String.fromCodePoint(Number.parseInt(code, 16));
    if (/^[\r\n\u2028\u2029]/.test(other)) return '';
    return SINGLE_ESCAPES[other] ?? other;
  });
}

const isLiteral = (token) => token?.kind === 'string' || token?.kind === 'template';

/** Where a plain operand (`a`, `a.b`, `a?.b(c)`, `a[0]`, `1`) that starts at `start` ends, or -1. */
function operandEnd(tokens, start) {
  let index = start;
  if (!['word', 'number'].includes(tokens[index]?.kind)) return -1;
  index += 1;
  for (;;) {
    const text = tokens[index]?.text;
    if ((text === '.' || text === '?.') && tokens[index + 1]?.kind === 'word') index += 2;
    else if ((text === '(' || text === '[') && tokens[index]?.kind === 'punct') index = closing(tokens, index) + 1;
    else return index;
  }
}

/**
 * Every literal text of `tokens` as `{ at, text }`: a literal, or a chain
 * of them joined with `+`, and, on their own, the literals inside each
 * template's expressions.
 */
function literalChains(tokens) {
  const chains = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (!isLiteral(token)) continue;
    const parts = [token];
    let next = index + 1;
    for (;;) {
      if (tokens[next]?.text !== '+') break;
      if (isLiteral(tokens[next + 1])) {
        parts.push(tokens[next + 1]);
        next += 2;
        continue;
      }
      const end = operandEnd(tokens, next + 1);
      if (end < 0 || tokens[end]?.text !== '+' || !isLiteral(tokens[end + 1])) break;
      parts.push(null, tokens[end + 1]);
      next = end + 2;
    }
    chains.push({ at: token.at, text: parts.map((part) => (part ? cooked(part.text) : '${}')).join('') });
    for (const part of parts) {
      for (const expression of part?.expressions ?? []) chains.push(...literalChains(expression));
    }
    index = next - 1;
  }
  return chains;
}

/** The phrases among the literals of `code`, as `{ line, literal }`, and how many literals were read. */
function phrasesIn(code) {
  const chains = literalChains(tokenize(code).tokens);
  const phrases = chains
    .filter(({ text }) => isPhrase(text))
    .map(({ at, text }) => ({ line: lineOf(code, at), literal: text }));
  return { phrases, read: chains.length };
}

/** The popup's scripts under `src/`, by path relative to it: `[path, source]`. */
function popupScripts() {
  const root = new URL('../../src/', import.meta.url);
  return readdirSync(root, { recursive: true })
    .map((path) => String(path).replaceAll('\\', '/'))
    .filter((path) => /\.[cm]?js$/.test(path))
    .sort()
    .map((path) => [path, readFileSync(new URL(path, root), 'utf8')]);
}

test('no natural-language literal outside the locale files', () => {
  const scripts = popupScripts();
  const scanned = scripts.filter(([path]) => !isLanguageFile(path));
  const skipped = scripts.filter(([path]) => isLanguageFile(path)).map(([path]) => path);

  assert.deepEqual(skipped, ['demo-data.js', 'i18n/en.js', 'i18n/pt-BR.js'], 'files left out');
  const names = scanned.map(([path]) => path);
  const mustRead = ['app.js', 'bridge.js', 'debounce.js', 'dropdown.js', 'i18n/index.js', 'icons.js', 'view-model.js'];
  for (const expected of mustRead) {
    assert.ok(names.includes(expected), `${expected} was read`);
  }
  const found = [];
  const allowed = new Set();
  let read = 0;
  for (const [path, code] of scanned) {
    const scan = phrasesIn(code);
    read += scan.read;
    for (const { line, literal } of scan.phrases) {
      const entry = NOT_LANGUAGE.find((candidate) => candidate.file === path && candidate.literal === literal);
      if (entry) allowed.add(entry);
      else found.push(`${path}:${line} ${JSON.stringify(literal)}`);
    }
  }

  assert.ok(read >= 500, `only ${read} literals read`);
  assert.deepEqual(found, [], 'phrases a translation never reaches');
  assert.deepEqual(
    NOT_LANGUAGE.filter((entry) => !allowed.has(entry)).map(({ file, literal }) => `${file} ${literal}`),
    [],
    'allowed literals no longer there',
  );
});

test('the phrase scan reads every literal form, and only literals', () => {
  const code = [
    "const note = input ? t('confirm.recover.input') : 'Some monitors only undo this from their own buttons.';",
    "setText(ui.probeNote, t('more.probeEmpty') + ' (nothing else to try)');",
    'const suffix = `Updated ${count} monitors`;',
    'const nested = `${busy ? `Saving ${what} now` : t(`value.${slug}`)}`;',
    "const glued = 'Try' + ' again';",
    "const around = 'Wait ' + delay.seconds + ' seconds';",
    "const escaped = 'Hidden\\u0020text' + \"\\x20\";",
    "const contraction = \"It's broken\";",
    "const oneWord = 'Brightness';",
    "const key = t('confirm.body.generic', { feature: label, to: 'x' });",
    "const pair = ['0 0 24 24', 'M15.25 15.25 19.75 19.75', 'a b c'];",
    "// const inComment = 'In a comment';",
    '/* const inBlock = "In a block"; */',
    'const regex = /Some words here/u;',
    'const gap = `${first}${second}`;',
  ].join('\n');

  const { phrases } = phrasesIn(code);

  assert.deepEqual(
    phrases.map(({ line, literal }) => `${line} ${JSON.stringify(literal)}`),
    [
      '1 "Some monitors only undo this from their own buttons."',
      '2 " (nothing else to try)"',
      '3 "Updated ${} monitors"',
      '4 "Saving ${} now"',
      '5 "Try again"',
      '6 "Wait ${} seconds"',
      '7 "Hidden text "',
      '8 "It\'s broken"',
    ],
  );
});

// ------------------------------------------------ the demo's data, one reader
//
// D-2026-09-27-tray-app-9 leaves `demo-data.js` out of the phrase scan
// because its messages are the core's, and the demo bridge hands them to
// the popup as the backend's own answer. That holds only while the bridge
// is the one module that reads it: a sentence exported from there and
// shown by the popup would get past the scan and every translation. A
// module can only reach it by naming it in a literal — `import … from`,
// `import '…'`, `export … from`, `import(…)` — so a script under `src/`
// that has a literal naming `demo-data`, read as the phrase scan reads
// them (joined with `+`, escapes as characters, inside a template's
// expressions too), counts as one of its readers.

/** The one module of `src/` that may import `demo-data.js`. */
const DEMO_DATA_READER = 'bridge.js';

/** The literals of `code` that name `demo-data`, as `{ line, literal }`. */
function demoDataMentions(code) {
  return literalChains(tokenize(code).tokens)
    .filter(({ text }) => text.includes('demo-data'))
    .map(({ at, text }) => ({ line: lineOf(code, at), literal: text }));
}

test('only bridge.js imports demo-data.js', () => {
  const mentions = popupScripts().flatMap(([path, code]) =>
    demoDataMentions(code).map(({ line, literal }) => ({ path, line, literal })),
  );
  const readers = [...new Set(mentions.map(({ path }) => path))];

  assert.ok(readers.includes(DEMO_DATA_READER), `${DEMO_DATA_READER} is seen importing demo-data.js`);
  assert.deepEqual(
    mentions
      .filter(({ path }) => path !== DEMO_DATA_READER)
      .map(({ path, line, literal }) => `${path}:${line} ${JSON.stringify(literal)}`),
    [],
    `modules other than ${DEMO_DATA_READER} that import demo-data.js`,
  );
});

test('the demo-data scan finds each way to name the module, and only literals', () => {
  const code = [
    "import { DEMO_MESSAGES } from './demo-data.js';",
    'import * as demo from "demo-data.js";',
    "import '../demo-data.js';",
    "export { FAILURES } from './demo-data.js';",
    "const lazy = await import('./demo-data.js');",
    'const late = import(`./demo-data.js`);',
    "const glued = import('./demo-' + 'data.js');",
    "const hidden = import('./demo\\x2ddata.js');",
    "const nested = import(`./${'demo-data'}.js`);",
    "// import './demo-data.js';",
    "/* const commented = import('./demo-data.js'); */",
    "import { t } from './i18n/index.js';",
    "const other = await import('./demo.js');",
  ].join('\n');

  assert.deepEqual(
    demoDataMentions(code).map(({ line, literal }) => `${line} ${JSON.stringify(literal)}`),
    [
      '1 "./demo-data.js"',
      '2 "demo-data.js"',
      '3 "../demo-data.js"',
      '4 "./demo-data.js"',
      '5 "./demo-data.js"',
      '6 "./demo-data.js"',
      '7 "./demo-data.js"',
      '8 "./demo-data.js"',
      '9 "demo-data"',
    ],
  );
});
