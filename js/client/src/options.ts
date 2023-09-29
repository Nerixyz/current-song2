let prevOptions: string[] = [];

function clearPrevOptions(el: HTMLElement) {
  for (const opt of prevOptions) {
    el.removeAttribute(opt);
  }
  prevOptions = [];
}

function searchParams(source: string) {
  try {
    return new URLSearchParams(source.substring(1));
  } catch (e) {
    console.warn('Failed to parse search', e);
    return new URLSearchParams();
  }
}

function applyOptions(el: HTMLElement) {
  clearPrevOptions(el);

  const query = searchParams(location.search);
  const hash = searchParams(location.hash);

  const addOpt = (prefix: string, key: string, value: string) => {
    const name = `data-${prefix}-${key}`;
    el.setAttribute(name, value);
    prevOptions.push(name);
  };

  for (const [key, value] of query.entries()) {
    addOpt('query', key, value);
  }
  for (const [key, value] of hash.entries()) {
    addOpt('hash', key, value);
  }

  const position = parsePositionOption(query, hash);
  if (position.v) {
    addOpt('cs', 'vpos', position.v);
  }
  if (position.h) {
    addOpt('cs', 'hpos', position.h);
  }
}

export function setupOptions() {
  applyOptions(document.documentElement);
  addEventListener('hashchange', () => applyOptions(document.documentElement));
}

type VPos = 'top' | 'bottom' | 'center';
type HPos = 'right' | 'left' | 'center';
type Pos = { v: VPos | null; h: HPos | null };

function parsePositionOption(query: URLSearchParams, hash: URLSearchParams): Pos {
  const opt = query.get('position') ?? query.get('pos') ?? hash.get('position') ?? hash.get('pos');
  if (!opt) {
    return { v: null, h: null };
  }
  const pos: Pos = { v: null, h: null };
  const parseSingle = (c: string) => {
    switch (c.toLowerCase()) {
      case 'right':
      case 'r':
        pos.h = 'right';
        break;
      case 'left':
      case 'l':
        pos.h = 'left';
        break;
      case 'top':
      case 't':
        pos.v = 'top';
        break;
      case 'bottom':
      case 'b':
        pos.v = 'bottom';
        break;
      case 'center':
      case 'c':
        pos.v = 'center';
        pos.h = 'center';
        break;
      case 'vcenter':
        pos.v = 'center';
        break;
      case 'hcenter':
        pos.h = 'center';
        break;
    }
  };
  if (opt.length === 1) {
    parseSingle(opt);
  } else if (opt.length == 2) {
    parseSingle(opt[0]);
    parseSingle(opt[1]);
  } else {
    for (const part of opt.split('-')) {
      parseSingle(part);
    }
  }

  return pos;
}
