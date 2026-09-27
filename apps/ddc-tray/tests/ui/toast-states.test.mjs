import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';

// A toast tells a failure, and its text shows in no other state: a
// literal glued to it reaches the user unless the pseudo-locale checks a
// state that shows it (D-2026-09-27-tray-app-7). So every mention of
// `showToast` in the popup's scripts needs a state of its own, listed in
// the pseudo-locale spec's `TOAST_STATES`. The spec is read as text:
// importing it would run Playwright.

const SRC = new URL('../../src/', import.meta.url);
const SPEC = readFileSync(new URL('../e2e/pseudo-locale.spec.mjs', import.meta.url), 'utf8');

/** Every script of the popup, `src/` and below, as `{ file, code }`. */
function scripts(dir = SRC, prefix = '') {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const file = `${prefix}${entry.name}`;
    if (entry.isDirectory()) return scripts(new URL(`${entry.name}/`, dir), `${file}/`);
    if (!/\.(m|c)?js$/.test(entry.name)) return [];
    return [{ file, code: readFileSync(new URL(entry.name, dir), 'utf8') }];
  });
}

const DECLARATION = /^[ \t]*(?:export[ \t]+)?(?:async[ \t]+)?function\*?[ \t]*([\w$]+)[ \t]*\(/gm;

/**
 * Where each mention of `name` in the scripts sits, as `file › function`:
 * the last function declared before it, or `(module)`; the declaration of
 * `name` itself is not a mention.
 */
function mentions(name) {
  const found = [];
  for (const { file, code } of scripts()) {
    const functions = [...code.matchAll(DECLARATION)];
    for (const mention of code.matchAll(new RegExp(`\\b${name}\\b`, 'g'))) {
      const enclosing = functions.filter((declared) => declared.index <= mention.index).at(-1);
      const declaring = enclosing?.[1] === name && enclosing.index + enclosing[0].length > mention.index;
      if (!declaring) found.push(`${file} › ${enclosing?.[1] ?? '(module)'}`);
    }
  }
  return found.sort();
}

/** The lines between `open` and the first line that is `close`, in the spec. */
function block(open, close) {
  const lines = SPEC.split('\n');
  const start = lines.indexOf(open);
  assert.notEqual(start, -1, `the spec has no line ${JSON.stringify(open)}`);
  const end = lines.indexOf(close, start);
  assert.notEqual(end, -1, `nothing closes ${JSON.stringify(open)}`);
  return lines.slice(start + 1, end);
}

const ENTRY = /^ {2}\{ site: '([^']+)', state: '([^']+)' \},$/;

/** `TOAST_STATES` as written: every line of it must be one plain entry. */
function toastStates() {
  return block('const TOAST_STATES = [', '];').map((line) => {
    const entry = ENTRY.exec(line);
    assert.ok(entry, `TOAST_STATES line is not { site: '…', state: '…' }: ${JSON.stringify(line)}`);
    return { site: entry[1], state: entry[2] };
  });
}

/** The names of the states the spec checks. */
function stateNames() {
  return block('const STATES = [', '];').flatMap((line) => {
    const name = /^ {4}name: '([^']+)',$/.exec(line) ?? /^ {2}\{ name: '([^']+)',/.exec(line);
    return name ? [name[1]] : [];
  });
}

test('every toast call site has a pseudo-locale state', () => {
  const listed = toastStates();
  const names = stateNames();
  const states = listed.map(({ state }) => state);

  assert.deepEqual(
    mentions('showToast'),
    listed.map(({ site }) => site).sort(),
    'showToast mentions in src/ (left) against TOAST_STATES in the pseudo-locale spec (right)',
  );
  assert.ok(listed.length >= 4, `only ${listed.length} toast sites listed`);
  assert.deepEqual(
    states.filter((state) => !names.includes(state)),
    [],
    'TOAST_STATES names a state the spec does not check',
  );
  assert.equal(new Set(states).size, states.length, 'two toast sites share one state');
});

test("only showToast writes the toast's text", () => {
  assert.deepEqual(mentions('toastText'), ['app.js › (module)', 'app.js › showToast']);
  assert.deepEqual(
    scripts()
      .filter(({ code }) => code.includes('toast-text'))
      .map(({ file, code }) => [file, code.match(/toast-text/g).length]),
    [['app.js', 1]],
  );
});
