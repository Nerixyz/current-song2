import { BrowserTab, BrowserWindow, WindowId } from '../../src/types/tab.types';

export function mockWindow(
  id: WindowId,
  tabs: BrowserTab[],
  other: Partial<Exclude<BrowserWindow, 'id' | 'tabs'>> = {},
): BrowserWindow {
  if (tabs.length === 0) throw new Error('No tabs in window');
  let anyActive = false;
  for (const tab of tabs) {
    tab.windowId = id;
    if (tab.active) {
      if (anyActive) tab.active = false;
      else anyActive = true;
    }
  }
  if (!anyActive) tabs[0].active = true;
  return {
    id,
    incognito: false,
    focused: false,
    state: 'normal',
    alwaysOnTop: false,
    tabs,
    ...other,
  };
}
