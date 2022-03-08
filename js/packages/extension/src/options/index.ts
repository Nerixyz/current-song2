import StorageChange = browser.storage.StorageChange;

export enum Option {
  UseLegacyApi = 'use-legacy-api',
  FilterMode = 'filter-mode',
  Filters = 'filters',
}

export enum FilterMode {
  Allow = 'allow',
  Block = 'block',
}
export const DEFAULT_FILTER_MODE = FilterMode.Block;

export function listenOption<T>(name: Option, cb: (value: T | undefined) => void): () => void {
  const listener = (changes: { [p: string]: StorageChange }, areaName: string) => {
    if (areaName !== 'local' || !(name in changes)) return;

    cb(changes[name].newValue);
  };
  browser.storage.onChanged.addListener(listener);

  // This is important for the Connection class
  browser.storage.local.get(name).then(res => {
    console.log(name, res);
    cb(res[name]);
  });

  return () => {
    browser.storage.onChanged.removeListener(listener);
  };
}

export function listenJsonOption<T>(name: Option, cb: (value: T | undefined) => void) {
  return listenOption<string>(name, value => cb(value ? JSON.parse(value) : undefined));
}

export async function setOption(name: Option, value: unknown) {
  return await browser.storage.local.set({ [name]: value });
}

export function setJsonOption(name: Option, value: unknown) {
  return setOption(name, JSON.stringify(value));
}
