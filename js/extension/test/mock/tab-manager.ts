import { IBrowserInterface } from '../../src/background/BrowserInterface';
import { TabManager } from '../../src/background/TabManager';
import { BrowserTab, BrowserWindow, TabId } from '../../src/types/tab.types';
import { TabModel } from '../../src/background/TabModel';
import { IFilterStorage } from '../../src/filters/FilterStorage';
import { FilterManager } from '../../src/filters/FilterManager';
import { staticStorage } from './filter-storage';
import { FilterMode } from '../../src/options';

type UpdateCallback = (message: TabModel | null) => void;
export function mockTabManager({
  browser,
  initialTabs = [],
  initialWindows = [],
  filterStorage,
}: {
  browser: IBrowserInterface;
  initialTabs?: BrowserTab[];
  initialWindows?: BrowserWindow[];
  filterStorage?: IFilterStorage;
}) {
  let resolveFn: UpdateCallback | null = null;
  const nextUpdate = () => new Promise<TabModel | null>(res => (resolveFn = res));

  return {
    manager: new TabManager({
      initialTabs,
      initialWindows,
      browser,
      filterManager: new FilterManager(filterStorage ?? staticStorage([], FilterMode.Block, false)),
      updateCallback: msg => {
        if (!resolveFn) throw new Error('Unhandled event');
        resolveFn(msg);
        resolveFn = null;
      },
    }),
    nextUpdate,
  };
}

export async function expectTab(promise: Promise<TabModel | null>, id: TabId) {
  const tab = await promise;
  expect(tab).not.toBeNull();
  expect(tab!.id).toBe(id);
  return tab!;
}

export async function expectNotTabs(promise: Promise<TabModel | null>, ids: TabId[]) {
  const tab = await promise;
  expect(tab).not.toBeNull();
  expect(ids.every(id => tab!.id !== id)).toBe(true);
}

export async function expectNothing(promise: Promise<TabModel | null>) {
  const tab = await promise;
  expect(tab).toBeNull();
}
