import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import en from '../../src/i18n/en.js';
import ptBR from '../../src/i18n/pt-BR.js';
import {
  LOCALES,
  detectLocale,
  errorKey,
  featureKey,
  resolveLocale,
  t,
  translateOr,
  translator,
  valueKey,
} from '../../src/i18n/index.js';

// The core's catalog is the source of every alias and value name the UI
// labels; reading it here checks the keys against it without a JS copy.
const catalog = readFileSync(
  new URL('../../../../crates/ddc-core/src/domain/mccs_catalog.rs', import.meta.url),
  'utf8',
);
const rows = [
  ...catalog.matchAll(
    /row\(0x[0-9A-F]{2},\s*(?:Some\("[^"]*"\)|None),\s*(?:Some\("([^"]+)"\)|None),\s*\w+,\s*(\w+),/g,
  ),
];
const aliases = rows.filter(([, alias]) => alias).map(([, alias]) => alias);
const writableAliases = rows.filter(([, alias, access]) => alias && access === 'RW').map(([, alias]) => alias);
const valueNames = [...catalog.matchAll(/\(0x[0-9A-F]{2}, "([^"]+)"\)/g)].map(([, name]) => name);

const ERROR_KINDS = [
  'not_found',
  'unsupported',
  'invalid_value',
  'needs_confirmation',
  'timeout',
  'transport',
  'backend_unavailable',
  'unknown',
];

const keysWith = (prefix) => Object.keys(en).filter((key) => key.startsWith(prefix));
const placeholders = (text) => [...text.matchAll(/\{(\w+)\}/g)].map(([, name]) => name).sort();

test('the catalog of the core was read', () => {
  assert.equal(rows.length, 39);
  assert.ok(aliases.includes('brightness'));
  assert.ok(valueNames.includes('Off (DPM)'));
});

test('en and pt-BR have exactly the same keys', () => {
  assert.deepEqual(Object.keys(ptBR).sort(), Object.keys(en).sort());
  assert.deepEqual(Object.keys(LOCALES), ['en', 'pt-BR']);
});

test('no text is empty in either locale', () => {
  for (const [locale, texts] of Object.entries(LOCALES)) {
    const empty = Object.entries(texts).filter(([, text]) => typeof text !== 'string' || text.trim() === '');
    assert.deepEqual(empty, [], `empty texts in ${locale}`);
  }
});

test('both locales use the same placeholders in each text', () => {
  const mismatched = Object.keys(en).filter(
    (key) => placeholders(en[key]).join() !== placeholders(ptBR[key]).join(),
  );
  assert.deepEqual(mismatched, []);
});

test('every error kind of the contract has a text', () => {
  const missing = ERROR_KINDS.map(errorKey).filter((key) => !(key in en));
  assert.deepEqual(missing, []);
  assert.deepEqual(keysWith('error.').sort(), ERROR_KINDS.map(errorKey).sort());
});

test('feature labels are keyed by the aliases of the core and cover every writable one', () => {
  const labelled = keysWith('feature.');
  assert.deepEqual(
    labelled.filter((key) => !aliases.map(featureKey).includes(key)),
    [],
    'labels without a catalog alias',
  );
  assert.deepEqual(
    writableAliases.map(featureKey).filter((key) => !labelled.includes(key)),
    [],
    'writable aliases without a label',
  );
});

test('value labels are keyed by slugs of value names of the core', () => {
  const slugs = valueNames.map(valueKey);
  assert.deepEqual(
    keysWith('value.').filter((key) => !slugs.includes(key)),
    [],
  );
});

test('value keys are the slugs of the core names', () => {
  assert.equal(valueKey('Off (DPM)'), 'value.off-dpm');
  assert.equal(valueKey('Off (write-only)'), 'value.off-write-only');
  assert.equal(valueKey('Chinese (traditional)'), 'value.chinese-traditional');
  assert.equal(valueKey('5000 K'), 'value.5000-k');
  assert.equal(valueKey('DisplayPort-1'), 'value.displayport-1');
  assert.equal(featureKey('osd-lock'), 'feature.osd-lock');
});

test('pt tags resolve to pt-BR and anything else to en', () => {
  const cases = [
    ['pt-BR', 'pt-BR'],
    ['pt', 'pt-BR'],
    ['pt-PT', 'pt-BR'],
    ['PT_br', 'pt-BR'],
    ['en-US', 'en'],
    ['de-DE', 'en'],
    ['', 'en'],
    [undefined, 'en'],
    [['de-DE', 'pt-BR', 'en'], 'pt-BR'],
    [['en-GB', 'pt-BR'], 'en'],
    [[], 'en'],
  ];
  for (const [tags, expected] of cases) {
    assert.equal(resolveLocale(tags), expected, `resolveLocale(${JSON.stringify(tags)})`);
  }
});

test('the detected locale prefers the language list over the single language', () => {
  assert.equal(detectLocale({ languages: ['pt-BR'], language: 'en-US' }), 'pt-BR');
  assert.equal(detectLocale({ languages: [], language: 'pt-BR' }), 'pt-BR');
  assert.equal(detectLocale({ language: 'fr-FR' }), 'en');
  assert.equal(detectLocale(undefined), 'en');
});

test('t falls back to en, then to the key itself', () => {
  const dictionaries = { en: { greet: 'Hello', english: 'Only in en' }, 'pt-BR': { greet: 'Olá' } };

  assert.equal(translator('pt-BR', dictionaries)('greet'), 'Olá');
  assert.equal(translator('pt-BR', dictionaries)('english'), 'Only in en');
  assert.equal(translator('pt-BR', dictionaries)('nowhere'), 'nowhere');
  assert.equal(translator('fr', dictionaries)('greet'), 'Hello');
});

test('placeholders are filled from params and kept when a param is missing', () => {
  assert.equal(t('format.fraction', { current: 3, max: 10 }, 'pt-BR'), '3 de 10');
  assert.equal(t('format.fraction', { current: 3, max: 10 }), '3 of 10');
  assert.equal(t('format.fraction', { max: 10 }, 'pt-BR'), '{current} de 10');
  assert.equal(t('feature.brightness', {}, 'pt-BR'), 'Brilho');
});

test('translateOr gives the fallback only when no locale has the key', () => {
  const pt = translator('pt-BR');

  assert.equal(translateOr(pt, valueKey('Off (DPM)'), 'Off (DPM)'), 'Em espera (DPM)');
  assert.equal(translateOr(pt, valueKey('HDMI-1'), 'HDMI-1'), 'HDMI-1');
});
