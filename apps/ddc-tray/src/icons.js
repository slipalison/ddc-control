// The popup's own icons (D-2026-09-26-tray-app-5), kept as data and built
// as inline SVG through the DOM: no markup string is ever parsed and no
// inline style is needed under the CSP (D-2026-09-26-tray-app-7). Line
// icons use the current text color; `brand` is the app icon and `empty` the
// no-monitor illustration, both in the app's cyan→blue→violet gradient.

const SVG_NS = 'http://www.w3.org/2000/svg';

const LINE = Object.freeze({
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '1.75',
  'stroke-linecap': 'round',
  'stroke-linejoin': 'round',
});

const BRAND_STOPS = Object.freeze([
  ['0', '#22D3EE'],
  ['0.5', '#3B82F6'],
  ['1', '#8B5CF6'],
]);

const line = (...shapes) => Object.freeze({ viewBox: '0 0 24 24', attrs: LINE, gradients: [], shapes });

/**
 * Every icon by name: a view box, attributes of the root `<svg>`, gradients
 * (`url(#name)` in a shape names one) and the shapes as `[tag, attrs]`.
 */
export const ICONS = Object.freeze({
  sun: line(
    ['circle', { cx: 12, cy: 12, r: 4.25 }],
    [
      'path',
      {
        d: 'M12 2.75v2M12 19.25v2M2.75 12h2M19.25 12h2M5.46 5.46l1.42 1.42M17.12 17.12l1.42 1.42M5.46 18.54l1.42-1.42M17.12 6.88l1.42-1.42',
      },
    ],
  ),
  contrast: line(
    ['circle', { cx: 12, cy: 12, r: 8.25 }],
    ['path', { d: 'M12 3.75a8.25 8.25 0 0 1 0 16.5z', fill: 'currentColor', stroke: 'none' }],
  ),
  volume: line(
    ['path', { d: 'M4.75 9.25h3l4.5-3.75v13l-4.5-3.75h-3a1 1 0 0 1-1-1v-3.5a1 1 0 0 1 1-1z' }],
    ['path', { d: 'M15.5 9.5a3.5 3.5 0 0 1 0 5M18 7a7 7 0 0 1 0 10' }],
  ),
  input: line(
    ['path', { d: 'M13.75 4.75h3.5a2 2 0 0 1 2 2v10.5a2 2 0 0 1-2 2h-3.5' }],
    ['path', { d: 'M4.25 12h10.25M11 8.25 14.75 12 11 15.75' }],
  ),
  droplet: line(
    ['path', { d: 'M12 3.75C9.5 6.9 6.25 10.4 6.25 13.75a5.75 5.75 0 0 0 11.5 0c0-3.35-3.25-6.85-5.75-10z' }],
    ['path', { d: 'M9.25 14a2.75 2.75 0 0 0 2.75 2.75' }],
  ),
  power: line(['path', { d: 'M12 3.75v7.5M7.05 6.4a7.5 7.5 0 1 0 9.9 0' }]),
  refresh: line(['path', { d: 'M19 12a7 7 0 1 1-2.05-4.95L19 9M19 4.75V9h-4.25' }]),
  check: line(['path', { d: 'M5.5 12.5l4 4L18.5 7.5', 'stroke-width': '2.5' }]),
  chevron: line(['path', { d: 'M6.75 9.5 12 14.75l5.25-5.25' }]),
  alert: line(
    [
      'path',
      {
        d: 'M10.27 4.99 3.28 17.1a2 2 0 0 0 1.73 3h13.98a2 2 0 0 0 1.73-3L13.73 4.99a2 2 0 0 0-3.46 0z',
      },
    ],
    ['path', { d: 'M12 9.25v4.25' }],
    ['circle', { cx: 12, cy: 16.75, r: 1, fill: 'currentColor', stroke: 'none' }],
  ),
  sliders: line(
    ['path', { d: 'M4 7.5h8.5M16.5 7.5H20M4 16.5h3.5M11.5 16.5H20' }],
    ['circle', { cx: 14.5, cy: 7.5, r: 2 }],
    ['circle', { cx: 9.5, cy: 16.5, r: 2 }],
  ),
  search: line(['circle', { cx: 10.75, cy: 10.75, r: 6 }], ['path', { d: 'M15.25 15.25 19.75 19.75' }]),
  brand: Object.freeze({
    viewBox: '0 0 256 256',
    attrs: {},
    gradients: [{ name: 'screen', attrs: { x1: 0, y1: 0, x2: 1, y2: 1 }, stops: BRAND_STOPS }],
    shapes: [
      ['path', { d: 'M108 184h40l8 30h-56z', fill: '#7383A0' }],
      ['rect', { x: 60, y: 206, width: 136, height: 26, rx: 13, fill: '#7383A0' }],
      ['rect', { x: 12, y: 24, width: 232, height: 168, rx: 28, fill: 'url(#screen)' }],
      ['circle', { cx: 128, cy: 108, r: 29, fill: '#FFFFFF' }],
      [
        'path',
        {
          d: 'M174 108L188 108M160.5 140.5L170.4 150.4M128 154L128 168M95.5 140.5L85.6 150.4M82 108L68 108M95.5 75.5L85.6 65.6M128 62L128 48M160.5 75.5L170.4 65.6',
          stroke: '#FFFFFF',
          'stroke-width': 17,
          'stroke-linecap': 'round',
          fill: 'none',
        },
      ],
    ],
  }),
  empty: Object.freeze({
    viewBox: '0 0 120 100',
    attrs: {},
    gradients: [
      { name: 'glass', attrs: { x1: 0, y1: 0, x2: 1, y2: 1 }, stops: BRAND_STOPS },
      {
        name: 'lens',
        attrs: { x1: 44, y1: 24, x2: 78, y2: 58, gradientUnits: 'userSpaceOnUse' },
        stops: BRAND_STOPS,
      },
    ],
    shapes: [
      ['path', { d: 'M51 74h18l3 11H48z', fill: 'currentColor', 'fill-opacity': 0.28 }],
      ['rect', { x: 36, y: 84, width: 48, height: 7, rx: 3.5, fill: 'currentColor', 'fill-opacity': 0.28 }],
      [
        'rect',
        {
          x: 9,
          y: 7,
          width: 102,
          height: 67,
          rx: 11,
          fill: 'url(#glass)',
          'fill-opacity': 0.14,
          stroke: 'currentColor',
          'stroke-opacity': 0.5,
          'stroke-width': 2.5,
        },
      ],
      ['circle', { cx: 57, cy: 37, r: 12, fill: 'none', stroke: 'url(#lens)', 'stroke-width': 4 }],
      [
        'path',
        { d: 'M65.5 45.5 74 54', fill: 'none', stroke: 'url(#lens)', 'stroke-width': 5, 'stroke-linecap': 'round' },
      ],
    ],
  }),
});

let instances = 0;

/**
 * A new `<svg>` of icon `name`, hidden from assistive technology — the
 * control around it carries the text. Gradient ids are unique per call, so
 * the same icon can appear twice in one document.
 * @param {keyof typeof ICONS} name
 * @param {Document} [doc]
 * @returns {SVGSVGElement}
 */
export function createIcon(name, doc = globalThis.document) {
  const icon = ICONS[name];
  if (!icon) throw new Error(`unknown icon ${name}`);
  instances += 1;
  const ids = new Map(icon.gradients.map(({ name: gradient }) => [gradient, `icon-${instances}-${gradient}`]));
  const svg = node(doc, 'svg', {
    viewBox: icon.viewBox,
    'aria-hidden': 'true',
    focusable: 'false',
    class: `icon icon-${name}`,
    ...icon.attrs,
  });
  if (ids.size > 0) svg.append(defs(doc, icon.gradients, ids));
  for (const [tag, attrs] of icon.shapes) svg.append(node(doc, tag, withIds(attrs, ids)));
  return svg;
}

function defs(doc, gradients, ids) {
  const container = node(doc, 'defs', {});
  for (const { name, attrs, stops } of gradients) {
    const gradient = node(doc, 'linearGradient', { id: ids.get(name), ...attrs });
    for (const [offset, color] of stops) gradient.append(node(doc, 'stop', { offset, 'stop-color': color }));
    container.append(gradient);
  }
  return container;
}

function withIds(attrs, ids) {
  return Object.fromEntries(
    Object.entries(attrs).map(([key, value]) => [
      key,
      typeof value === 'string' ? value.replace(/url\(#(\w+)\)/, (ref, name) => `url(#${ids.get(name) ?? name})`) : value,
    ]),
  );
}

function node(doc, tag, attrs) {
  const element = doc.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) element.setAttribute(key, String(value));
  return element;
}
