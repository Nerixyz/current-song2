import { TabModel } from './TabModel';
import { BrowserTab, BrowserWindow, TabActivateInfo, TabId, WindowId } from '../types/tab.types';
import { filter, first } from '../utils/iterators';
import { MessageCreator } from '../types/message.types';
import { VideoMetadata, VideoPlayPosition } from '../types/video.types';
import { isChromeLike } from '../utils/chrome';

interface TabManagerOptions {
  initialTabs: BrowserTab[];
  initialWindows: BrowserWindow[];
  updateCallback: (message: MessageCreator | null) => void;
}

interface FindAndEmitOptions {
  forceSendIfActive?: boolean;
}

export class TabManager extends EventTarget {
  /** This includes tabs that are audible OR have metadata */
  readonly tabs = new Map<TabId, TabModel>();

  readonly blockedWindows = new Set<WindowId>();
  activeWindowId: number | null = null;

  sentTabId: number | null = null;

  private readonly isChrome = isChromeLike();
  private readonly updateCallback: (message: MessageCreator | null) => void;

  constructor({ initialTabs, initialWindows, updateCallback }: TabManagerOptions) {
    super();
    this.updateCallback = updateCallback;

    this.initListeners();

    for (const window of initialWindows) {
      if (window.focused && window.id !== undefined) this.activeWindowId = window.id;

      this.updateWindow(window);
    }

    for (const tab of initialTabs) {
      this.addTab(tab);
    }

    this.findAndEmitActiveTab();
  }

  setPlayPosition(tab: BrowserTab, position?: VideoPlayPosition) {
    const model = this.getOrCreate(tab);
    model.updateTimeline(position);

    if (this.sentTabId === model.id) this.updateCallback(model);
  }

  setMetadata(tab: BrowserTab, meta?: VideoMetadata) {
    const model = this.getOrCreate(tab);
    const anyChange = model.updateMetadata(meta);
    if (!anyChange) return;

    if (this.sentTabId === model.id) this.updateCallback(model);
  }

  private addTab(tab: BrowserTab): TabModel {
    const model = new TabModel(tab);
    this.tabs.set(tab.id ?? -1, model);
    return model;
  }

  private updateWindow(window: BrowserWindow) {
    if (window.state === 'fullscreen') this.blockedWindows.add(window.id ?? -1);
    else this.blockedWindows.delete(window.id ?? -1);
  }

  private getOrCreate(tab: BrowserTab): TabModel {
    const model = this.tabs.get(tab.id ?? -1);
    if (model) return model;
    return this.addTab(tab);
  }

  private initListeners() {
    browser.tabs.onCreated.addListener(this.tabCreated.bind(this));
    browser.tabs.onRemoved.addListener(this.tabRemoved.bind(this));
    browser.tabs.onUpdated.addListener(this.tabUpdated.bind(this));
    browser.tabs.onActivated.addListener(this.tabActivated.bind(this));

    // do not use windows api on chrome see: https://bugs.chromium.org/p/chromium/issues/detail?id=387377
    if (!this.isChrome) {
      browser.windows.onFocusChanged.addListener(this.windowFocused.bind(this));
      browser.windows.onRemoved.addListener(this.windowRemoved.bind(this));
    }
  }

  private tabCreated(tab: BrowserTab) {
    this.addTab(tab);
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

    const browserTab = await browser.tabs.get(tabId);
    const changed = tab.updateTabMeta(browserTab);
    if (!changed) return;

    this.findAndEmitActiveTab({ forceSendIfActive: browserTab.id === this.sentTabId });
  }

  private async windowFocused(windowId: WindowId) {
    if (!this.isChrome) {
      this.activeWindowId = windowId;

      for (const window of await browser.windows.getAll()) {
        this.updateWindow(window);
      }
      this.findAndEmitActiveTab();
    }
  }

  private windowRemoved(windowId: WindowId) {
    if (this.activeWindowId === windowId) this.activeWindowId = null;
    this.blockedWindows.delete(windowId);

    // don't update tabs here since there will be separate events for them
  }

  private findAndEmitActiveTab({ forceSendIfActive }: FindAndEmitOptions = {}) {
    const audible = first(filter(this.tabs.values(), x => this.isValidTab(x)));

    if (audible) {
      if (this.sentTabId === audible.id && !forceSendIfActive) return;

      this.sentTabId = audible.id;
    } else if (this.sentTabId) {
      this.sentTabId = null;
    } else {
      return;
    }

    this.updateCallback(audible);
  }

  private isValidTab(tab: TabModel): boolean {
    if (tab.active && this.activeWindowId === tab.windowId) return false;
    if (!tab.audible || tab.muted) return false;

    return !this.blockedWindows.has(tab.windowId);
  }
}
