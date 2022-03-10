import { BrowserTab, BrowserWindow, TabActivateInfo, TabId, WindowId } from '../types/tab.types';
import { isChromeLike } from '../utils/chrome';

export interface IBrowserInterface {
  // Promise will be rejected if this tab doesn't exist
  getTab(id: TabId): Promise<BrowserTab>;
  getAllWindows(): Promise<BrowserWindow[]>;

  addTabCreatedListener(cb: (tab: BrowserTab) => void): void;
  addTabRemovedListener(cb: (tabId: TabId) => void): void;
  addTabUpdatedListener(cb: (tabId: TabId) => void): void;
  addTabActivatedListener(cb: (info: TabActivateInfo) => void): void;
  addWindowFocusChangedListener(cb: (windowId: WindowId) => void): void;
  addWindowRemovedListener(cb: (windowId: WindowId) => void): void;
}

export const DefaultBrowserInterface: IBrowserInterface = {
  getTab(id) {
    return browser.tabs.get(id);
  },
  getAllWindows() {
    return browser.windows.getAll();
  },
  addTabActivatedListener(cb) {
    browser.tabs.onActivated.addListener(cb);
  },
  addTabCreatedListener(cb) {
    browser.tabs.onCreated.addListener(cb);
  },
  addTabRemovedListener(cb) {
    browser.tabs.onRemoved.addListener(cb);
  },
  addTabUpdatedListener(cb) {
    browser.tabs.onUpdated.addListener(cb);
  },
  // do not use windows api on chrome see: https://bugs.chromium.org/p/chromium/issues/detail?id=387377
  addWindowFocusChangedListener(cb) {
    if (!isChromeLike()) browser.windows.onFocusChanged.addListener(cb);
  },
  addWindowRemovedListener(cb) {
    if (!isChromeLike()) browser.windows.onRemoved.addListener(cb);
  },
};
