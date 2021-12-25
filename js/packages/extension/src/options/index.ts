export enum Option {
  UseLegacyApi = 'use-legacy-api',
}

export function listenOption<T>(name: Option, cb: (value: T) => void) {
  browser.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== 'local' || !(name in changes)) return;

    cb(changes[name].newValue);
  });

  // This is important for the Connection class
  browser.storage.local.get(name).then(res => cb(res[name]));
}

export async function setOption(name: Option, value: unknown) {
  return await browser.storage.local.set({ [name]: value });
}
