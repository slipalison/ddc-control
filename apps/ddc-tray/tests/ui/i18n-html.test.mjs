import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import en from '../../src/i18n/en.js';
import ptBR from '../../src/i18n/pt-BR.js';
import { ICONS } from '../../src/icons.js';

// Every text of the popup comes from a key (D-2026-09-26-tray-app-6), and
// under the strict CSP (D-2026-09-26-tray-app-7) nothing inline runs or
// styles it. These read the popup's sources as text: the page itself is
// checked in a browser by the Playwright suite.

const source = (path) => readFileSync(new URL(`../../src/${path}`, import.meta.url), 'utf8');

const html = source('index.html');
const app = source('app.js');
const viewModel = source('view-model.js');
const scripts = { 'app.js': app, 'view-model.js': viewModel, 'icons.js': source('icons.js') };

/** Attributes a person reads or hears: they may only come from keys. */
const READABLE_ATTRIBUTES = [
  'aria-label',
  'aria-description',
  'aria-roledescription',
  'aria-valuetext',
  'aria-placeholder',
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

test('a slider fill is set as a custom property, the one style the script touches', () => {
  const styleWrites = [...app.matchAll(/\.style\.(\w+)/g)].map(([, member]) => member);

  assert.deepEqual([...new Set(styleWrites)], ['setProperty']);
  assert.match(app, /style\.setProperty\('--fill'/);
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
