import StorageChange = browser.storage.StorageChange;

export enum Option {
  UseLegacyApi = 'use-legacy-api',
  FilterMode = 'filter-mode',
  Filters = 'filters',
  IncludeFocusedTabs = 'include-focused-tabs',
  ApiPort = 'api-port',
  ApiHost = 'api-host',
}

export enum FilterMode {
  Allow = 'allow',
  Block = 'block',
}

export const DEFAULT_FILTER_MODE = FilterMode.Block;
export const DEFAULT_INCLUDE_FOCUSED_TABS = false;
export const DEFAULT_LEGACY_PORT = 232;
export const DEFAULT_CURRENT_PORT = 48457;

/* istanbul ignore next */
export function listenOption<T>(name: Option, cb: (value: T | undefined) => void): () => void {
  const listener = (changes: { [p: string]: StorageChange }, areaName: string) => {
    if (areaName !== 'local' || !(name in changes)) return;

    cb(changes[name].newValue);
  };
  browser.storage.onChanged.addListener(listener);

  // This is important for the Connection class
  browser.storage.local.get(name).then(res => {
    cb(res[name]);
  });

  return () => {
    browser.storage.onChanged.removeListener(listener);
  };
}

/* istanbul ignore next */
export function listenJsonOption<T>(name: Option, cb: (value: T | undefined) => void) {
  return listenOption<string>(name, value => cb(value ? JSON.parse(value) : undefined));
}

/* istanbul ignore next */
export async function setOption(name: Option, value: unknown) {
  return await browser.storage.local.set({ [name]: value });
}

/* istanbul ignore next */
export function setJsonOption(name: Option, value: unknown) {
  return setOption(name, JSON.stringify(value));
}
