import { BrowserTab, BrowserWindow, TabActivateInfo, TabId, WindowId } from '../../src/types/tab.types';
import { IBrowserInterface } from '../../src/background/BrowserInterface';
import { mockWindow } from './window';
import { find } from '../iterators';
import { queueMicrotaskChain } from '../pseudo-async';

type Cb<T> = (value: T) => void;
export function mockBrowser(initialWindows: BrowserWindow[] = []) {
  initialWindows = initialWindows.filter(w => w.id && w.tabs?.length);
  const tabs = new Map(initialWindows.map(win => (win.tabs ?? []).map(t => [t.id ?? -1, t] as const)).flat());
  const windows = new Map(initialWindows.map(w => [w.id ?? -1, w]));
  let activeWindowId = find(windows.values(), w => w.focused)?.id ?? -1;
  const activeTabs = new Map(
    initialWindows.map(w => {
      const id = w.tabs?.find(t => t.active)?.id;
      if (typeof id !== 'number') throw new Error('No tab in window is active');
      return [w.id ?? -1, id];
    }),
  );

  let onTabCreated: Cb<BrowserTab> | null = null;
  let onTabRemoved: Cb<TabId> | null = null;
  let onTabUpdated: Cb<TabId> | null = null;
  let onTabActivated: Cb<TabActivateInfo> | null = null;
  let onWindowFocusChanged: Cb<WindowId> | null = null;
  let onWindowRemoved: Cb<WindowId> | null = null;

  const addTab = (tab: BrowserTab) => {
    if (typeof tab.id !== 'number' || typeof tab.windowId !== 'number') throw new Error('invalid tab');
    if (!tab.active) throw new Error("trying to add a tab but it's not active");
    if (tabs.has(tab.id)) throw new Error('Tab already exists');

    let focusWindowFlag: null | WindowId = null;
    let previousTabId: TabId | undefined = undefined;
    if (!windows.has(tab.windowId)) {
      windows.set(tab.windowId, mockWindow(tab.windowId, [tab]));
      focusWindowFlag = tab.windowId;
    } else {
      const win = windows.get(tab.windowId)!;
      if (!win.tabs?.length) throw new Error('window without tabs');
      win.tabs.push(tab);
      previousTabId = activeTabs.get(tab.windowId);
    }
    activeTabs.set(tab.windowId, tab.id);
    tabs.set(tab.id, tab);

    queueMicrotaskChain(
      () => onTabCreated?.(tab),
      () => onTabActivated?.({ tabId: tab.id ?? -1, windowId: tab.windowId ?? -1, previousTabId }),
      () => {
        if (typeof focusWindowFlag === 'number') focusWindow(focusWindowFlag);
      },
    );
  };
  const removeTab = (id: TabId) => {
    const tab = tabs.get(id);
    if (!tab) throw new Error('No such tab');
    const win = windows.get(tab.windowId ?? -1);
    if (!win || !win.tabs || win.tabs.length === 0) throw new Error('Tab without window or window without tabs');
    win.tabs = win.tabs.filter(t => t.id !== tab.id);

    queueMicrotask(() => {
      onTabRemoved?.(id);
      if (win.tabs?.length === 0) {
        windows.delete(win.id ?? -1);
        activeTabs.delete(win.id ?? -1);
        queueMicrotask(() => onWindowRemoved?.(win.id ?? -1));
      }
    });
  };
  const changeTab = (id: TabId, cb: Cb<BrowserTab>) => {
    const tab = tabs.get(id);
    if (!tab) throw new Error('No such tab');
    cb(tab);
    queueMicrotask(() => onTabUpdated?.(id));
  };
  const activateTab = (id: TabId) => {
    const tab = tabs.get(id);
    if (!tab) throw new Error("tab doesn't exist");
    const window = windows.get(tab.windowId ?? -1);
    if (!window) throw new Error('no such window');
    if (activeTabs.get(window.id ?? -1) === id) return;
    queueMicrotaskChain(
      () => changeTab(activeTabs.get(window.id ?? -1) ?? -1, tab => (tab.active = false)),
      () => {
        activeTabs.set(window.id ?? -1, id);
        changeTab(id, tab => (tab.active = true));
      },
      () => focusWindow(window.id ?? -1),
    );
  };

  const focusWindow = (id: WindowId) => {
    if (id === activeWindowId) return;
    const window = windows.get(id);
    if (windows.has(activeWindowId)) windows.get(activeWindowId)!.focused = false;
    if (window) window.focused = true;
    activeWindowId = id;
    queueMicrotask(() => onWindowFocusChanged?.(id));
  };
  const changeWindowState = (id: WindowId, state: Exclude<BrowserWindow['state'], undefined>) => {
    const window = windows.get(id);
    if (!window) throw new Error('invalid window');
    if (!window.focused) throw new Error('Only focused windows can change their state');
    window.state = state;
  };

  return {
    browser: {
      getAllWindows: () => Promise.resolve([...windows.values()]),
      getTab: id => (tabs.has(id) ? Promise.resolve(tabs.get(id)!) : Promise.reject('no tab')),

      addTabCreatedListener: (cb: (tab: BrowserTab) => void) => (onTabCreated = cb),
      addTabRemovedListener: (cb: (tabId: TabId) => void) => (onTabRemoved = cb),
      addTabUpdatedListener: (cb: (tabId: TabId) => void) => (onTabUpdated = cb),
      addTabActivatedListener: (cb: (info: TabActivateInfo) => void) => (onTabActivated = cb),
      addWindowFocusChangedListener: (cb: (windowId: WindowId) => void) => (onWindowFocusChanged = cb),
      addWindowRemovedListener: (cb: (windowId: WindowId) => void) => (onWindowRemoved = cb),
    } as IBrowserInterface,
    initialTabs: [...tabs.values()],
    initialWindows,
    addTab,
    removeTab,
    changeTab,
    activateTab,
    focusWindow,
    changeWindowState,
  };
}
