import { TabChange, TabModel } from './TabModel';
import { BrowserTab, BrowserWindow, TabActivateInfo, TabId, WindowId } from '../types/tab.types';
import { filter, first } from '../utils/iterators';
import { VideoMetadata, VideoPlayPosition } from '../types/video.types';
import { FilterManager } from '../filters/FilterManager';
import { IBrowserInterface } from './BrowserInterface';

interface TabManagerOptions {
  initialTabs: BrowserTab[];
  initialWindows: BrowserWindow[];
  updateCallback: (message: TabModel | null) => void;
  filterManager: FilterManager;
  browser: IBrowserInterface;
}

interface FindAndEmitOptions {
  forceSendIfActive?: boolean;
  forceSendIfNotActive?: boolean;
}

export class TabManager extends EventTarget {
  /** This includes tabs that are audible OR have metadata */
  readonly tabs = new Map<TabId, TabModel>();

  readonly blockedWindows = new Set<WindowId>();
  activeWindowId: number | null = null;

  sentTabId: number | null = null;

  private readonly updateCallback: (message: TabModel | null) => void;
  private readonly filterManager: FilterManager;
  private readonly browser: IBrowserInterface;

  constructor({ initialTabs, initialWindows, updateCallback, filterManager, browser }: TabManagerOptions) {
    super();
    this.updateCallback = updateCallback;
    this.browser = browser;
    this.filterManager = filterManager;
    this.filterManager.setUpdateListener(() => this.filtersUpdated());

    this.initListeners();

    for (const window of initialWindows) {
      if (window.focused && window.id !== undefined) this.activeWindowId = window.id;

      this.updateWindow(window);
    }

    for (const tab of initialTabs) {
      try {
        this.addTab(tab);
      } catch (e) {
        console.error(e, tab);
      }
    }

    // to make sure the update callback is valid
    queueMicrotask(() => this.findAndEmitActiveTab({ forceSendIfNotActive: true }));
  }

  setPlayPosition(tab: BrowserTab, position?: VideoPlayPosition) {
    if (!isId(tab.id)) return console.warn('Invalid tab', tab);

    const model = this.tabs.get(tab.id);
    if (!model) return console.warn('Tab not tracked:', tab);
    model.updateTimeline(position);

    if (this.sentTabId === model.id) this.updateCallback(model);
  }

  setMetadata(tab: BrowserTab, meta?: VideoMetadata) {
    if (!isId(tab.id)) return console.warn('Invalid tab', tab);

    const model = this.tabs.get(tab.id);
    if (!model) return console.warn('Tab not tracked:', tab);
    const anyChange = model.updateMetadata(meta);
    if (!anyChange) return;

    if (this.sentTabId === model.id) this.updateCallback(model);
  }

  private addTab(tab: BrowserTab): TabModel {
    if (!isId(tab.id)) throw new Error('Invalid tab');

    // This won't throw since we know there's a valid tab-id
    const model = new TabModel(tab);
    this.tabs.set(tab.id, model);
    return model;
  }

  private updateWindow(window: BrowserWindow) {
    if (!isId(window.id)) return console.warn('Invalid window', window);

    if (window.state === 'fullscreen') this.blockedWindows.add(window.id);
    else this.blockedWindows.delete(window.id);
  }

  private initListeners() {
    this.browser.addTabCreatedListener(this.tabCreated.bind(this));
    this.browser.addTabRemovedListener(this.tabRemoved.bind(this));
    this.browser.addTabUpdatedListener(this.tabUpdated.bind(this));
    this.browser.addTabActivatedListener(this.tabActivated.bind(this));
    this.browser.addWindowFocusChangedListener(this.windowFocused.bind(this));
    this.browser.addWindowRemovedListener(this.windowRemoved.bind(this));
  }

  private tabCreated(tab: BrowserTab) {
    try {
      this.addTab(tab);
    } catch (e) {
      console.error(e, tab);
    }
  }

  private tabRemoved(tabId: TabId) {
    this.tabs.delete(tabId);

    this.findAndEmitActiveTab();
  }

  private tabActivated(info: TabActivateInfo) {
    this.activeWindowId = info.windowId;

    if (info.previousTabId !== undefined) this.tabs.get(info.previousTabId)?.setActive(false);

    this.tabs.get(info.tabId)?.setActive(true);

    this.findAndEmitActiveTab();
  }

  private async tabUpdated(tabId: TabId) {
    const tab = this.tabs.get(tabId);
    if (!tab) return;

    const browserTab = await this.browser.getTab(tabId);
    const changeInfo = tab.updateTabMeta(browserTab);
    if (changeInfo === TabChange.NotChanged) return;
    else if (changeInfo === TabChange.UrlChanged) {
      // Do not send if active since nothing actually changed.
      // We still need to check for filters though.
      this.findAndEmitActiveTab();
    } else {
      this.findAndEmitActiveTab({ forceSendIfActive: browserTab.id === this.sentTabId });
    }
  }

  private async windowFocused(windowId: WindowId) {
    this.activeWindowId = windowId;

    for (const window of await this.browser.getAllWindows()) {
      this.updateWindow(window);
    }
    this.findAndEmitActiveTab();
  }

  private windowRemoved(windowId: WindowId) {
    if (this.activeWindowId === windowId) this.activeWindowId = null;
    this.blockedWindows.delete(windowId);

    // don't update tabs here since there will be separate events for them
  }

  private filtersUpdated() {
    this.findAndEmitActiveTab();
  }

  private findAndEmitActiveTab({ forceSendIfActive, forceSendIfNotActive }: FindAndEmitOptions = {}) {
    // optimization?: The last sent tab is still valid and exists, and we don't need to resend (forceSendIfActive).
    // !forceSendIfActive && this.sentTabId !== null && this.tabs.has(this.sentTabId) && this.isValidTab(this.tabs.get(this.sentTabId!)!)

    const audible = first(filter(this.tabs.values(), x => this.isValidTab(x)));

    if (audible) {
      if (this.sentTabId === audible.id && !forceSendIfActive) return;

      this.sentTabId = audible.id;
    } else if (this.sentTabId) {
      this.sentTabId = null;
    } else if (!forceSendIfNotActive) {
      return;
    }

    this.updateCallback(audible);
  }

  private isValidTab(tab: TabModel): boolean {
    if (!this.filterManager.includeFocusedTabs && tab.active && this.activeWindowId === tab.windowId) return false;
    if (!tab.audible || tab.muted) return false;
    if (this.blockedWindows.has(tab.windowId)) return false;

    return this.filterManager.checkUrl(tab.url);
  }
}

// Helper to check if a tab/window has a valid id
function isId(id: number | undefined): id is number {
  return typeof id === 'number';
}
