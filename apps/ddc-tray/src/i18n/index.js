// Every text the popup shows comes from a key (D-2026-09-26-tray-app-6):
// the locale's text, else the English one, else the key itself, so a
// missing text is visible instead of blank.

import en from './en.js';
import ptBR from './pt-BR.js';

export const DEFAULT_LOCALE = 'en';

export const LOCALES = Object.freeze({ en, 'pt-BR': ptBR });

/**
 * The supported locale of the first tag that names one: `pt*` is pt-BR,
 * `en*` is en. Anything else, or nothing, is en.
 * @param {string | readonly string[] | undefined} tags
 * @returns {'en' | 'pt-BR'}
 */
export function resolveLocale(tags) {
  const list = Array.isArray(tags) ? tags : [tags];
  for (const tag of list) {
    const lower = String(tag ?? '').toLowerCase();
    if (lower.startsWith('pt')) return 'pt-BR';
    if (lower.startsWith('en')) return 'en';
  }
  return DEFAULT_LOCALE;
}

/**
 * The locale the user's browser or webview asks for.
 * @param {{ languages?: readonly string[], language?: string } | undefined} nav
 */
export function detectLocale(nav) {
  return resolveLocale(nav?.languages?.length ? nav.languages : nav?.language);
}

/**
 * A `t(key, params)` bound to `locale`. `{name}` placeholders take
 * `params.name`; one without a param stays as written.
 * @param {string} locale
 * @param {Record<string, Record<string, string>>} [dictionaries]
 * @returns {(key: string, params?: Record<string, unknown>) => string}
 */
export function translator(locale, dictionaries = LOCALES) {
  const texts = dictionaries[locale] ?? {};
  const fallback = dictionaries[DEFAULT_LOCALE] ?? {};
  return (key, params = {}) => interpolate(texts[key] ?? fallback[key] ?? key, params);
}

/**
 * One-off translation of `key`.
 * @param {string} key
 * @param {Record<string, unknown>} [params]
 * @param {string} [locale]
 */
export function t(key, params = {}, locale = DEFAULT_LOCALE) {
  return translator(locale)(key, params);
}

/**
 * The text of `key`, or `fallback` when no locale has one — `t` answers
 * with the key itself in that case.
 * @param {(key: string, params?: Record<string, unknown>) => string} tr
 * @param {string} key
 * @param {string} fallback
 */
export function translateOr(tr, key, fallback) {
  const text = tr(key);
  return text === key ? fallback : text;
}

/**
 * The key of a value the core names `name`: `Off (DPM)` is `value.off-dpm`.
 * @param {string} name
 */
export function valueKey(name) {
  return `value.${slug(name)}`;
}

/**
 * The key of the label of a feature the core calls `alias`.
 * @param {string} alias
 */
export function featureKey(alias) {
  return `feature.${alias}`;
}

/**
 * The key of the text of an error of `kind`.
 * @param {string} kind
 */
export function errorKey(kind) {
  return `error.${kind}`;
}

function slug(name) {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function interpolate(text, params) {
  return text.replace(/\{(\w+)\}/g, (placeholder, name) =>
    Object.hasOwn(params, name) ? String(params[name]) : placeholder,
  );
}
