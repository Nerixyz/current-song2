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
}

export function setupOptions() {
  applyOptions(document.documentElement);
  addEventListener('hashchange', () => applyOptions(document.documentElement));
}
