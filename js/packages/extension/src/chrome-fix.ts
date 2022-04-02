(function fixChrome() {
  if (!globalThis.browser) {
    // @ts-ignore -- fix for chrome
    globalThis.browser = chrome;
  }
  tryPromisify(browser.tabs, 'get');
  tryPromisify(browser.windows, 'getCurrent');
  tryPromisify(browser.windows, 'getAll');
  tryPromisify(browser.storage.local, 'get');
  tryPromisify(browser.storage.local, 'set');
})();

export function tryPromisify<K extends string, T extends { [x in K]: (arg: any) => Promise<any> }>(obj: T, key: K) {
  if (obj === undefined) return;
  if (obj[key].length === 0) {
    // assume this is chrome
    const base = obj[key];
    // @ts-ignore -- wrong types or something, this is fine
    obj[key] = arg1 => new Promise(resolve => base.call(obj, arg1, resolve));
  }
}
