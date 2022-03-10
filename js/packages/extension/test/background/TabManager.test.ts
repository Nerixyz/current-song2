import { expectNothing, expectNotTabs, expectTab, mockTabManager } from '../mock/tab-manager';
import { mockBrowser } from '../mock/browser-interface';
import { mockWindow } from '../mock/window';
import { mockAudibleMutedTab, mockAudibleTab } from '../mock/tab';
import { dynamicStorage } from '../mock/filter-storage';
import { FilterMode } from '../../src/options';
import { waitNMicrotasks } from '../pseudo-async';

const make2Wind4TabBrowser = () =>
  mockBrowser([
    mockWindow(
      1,
      [
        mockAudibleTab(11, 1, { title: 'Aliens - Dancing', active: true }),
        mockAudibleTab(12, 1, { title: 'forsen - live' }),
        mockAudibleTab(13, 1, { title: 'nam - live' }),
        mockAudibleMutedTab(14, 1, { title: 'nerix - live' }),
      ],
      { focused: true },
    ),
    mockWindow(2, [
      mockAudibleTab(21, 2, { title: 'forsen - live' }),
      mockAudibleTab(22, 2, { title: 'forsen - live', active: true }),
      mockAudibleTab(23, 2, { title: 'forsen - live' }),
      mockAudibleMutedTab(24, 2, { title: 'forsen - live' }),
    ]),
  ]);

const make1Wind1TabBrowser = () =>
  mockBrowser([mockWindow(1, [mockAudibleTab(11, 1, { title: 'Aliens - Dancing', active: true })], { focused: true })]);

const make2Wind1TabBrowser = () =>
  mockBrowser([
    mockWindow(1, [mockAudibleTab(11, 1, { title: 'Aliens - Dancing', active: true })], { focused: true }),
    mockWindow(2, [mockAudibleTab(21, 2, { title: 'forsen - live', active: true })]),
  ]);

describe('TabManager', function () {
  it('should emit the active tab at the start', async function () {
    const { browser, initialTabs, initialWindows } = make2Wind4TabBrowser();
    const { nextUpdate } = mockTabManager({ browser, initialTabs, initialWindows });
    await expectNotTabs(nextUpdate(), [11, 14, 24]);
  });
  it('should emit when a tab is added and focus is changed', async function () {
    const { browser, initialTabs, initialWindows, addTab, activateTab } = make1Wind1TabBrowser();
    const { nextUpdate } = mockTabManager({ browser, initialTabs, initialWindows });
    await expectNothing(nextUpdate());
    addTab(mockAudibleTab(12, 1, { title: 'xd', active: true }));
    await expectTab(nextUpdate(), 11);
    activateTab(11);
    await expectTab(nextUpdate(), 12);
    activateTab(11);
    const promise = nextUpdate(); // start listening here, to make sure no duplicate events are emitted
    activateTab(12);
    await expectTab(promise, 11);
  });
  it('should emit when a window is added and focus+state is changed', async function () {
    const { browser, initialTabs, initialWindows, addTab, focusWindow, changeWindowState } = make1Wind1TabBrowser();
    const { nextUpdate } = mockTabManager({ browser, initialTabs, initialWindows });
    await expectNothing(nextUpdate());
    addTab(mockAudibleTab(22, 2, { title: 'xd', active: true }));
    await expectTab(nextUpdate(), 11);
    focusWindow(1);
    await expectTab(nextUpdate(), 22);
    focusWindow(2);
    await expectTab(nextUpdate(), 11);
    const promise = nextUpdate();
    // shouldn't emit since there's only one tab left to send, and it's already sent
    changeWindowState(2, 'fullscreen');
    focusWindow(1);
    await expectNothing(promise);
    focusWindow(-1);
    await expectTab(nextUpdate(), 11);
  });
  it('should react correctly when closing tabs/windows and changing focus', async function () {
    const { browser, initialTabs, initialWindows, removeTab, focusWindow, changeWindowState } = make2Wind1TabBrowser();
    const { nextUpdate } = mockTabManager({ browser, initialTabs, initialWindows });
    await expectTab(nextUpdate(), 21);
    removeTab(21);
    await expectNothing(nextUpdate());
    focusWindow(-1);
    await expectTab(nextUpdate(), 11);
    focusWindow(1);
    await expectNothing(nextUpdate());
    changeWindowState(1, 'fullscreen');
    const promise = nextUpdate();
    focusWindow(-1);
    focusWindow(1);
    changeWindowState(1, 'normal');
    focusWindow(-1);
    await expectTab(promise, 11);
  });
  describe('filters', function () {
    it('should block filtered urls', async function () {
      const { browser, initialTabs, initialWindows, changeTab } = make2Wind1TabBrowser();
      const { setFilters, setMode, filterStorage } = dynamicStorage(
        [{ value: 'github.com', isRegex: false }],
        FilterMode.Block,
      );
      const { nextUpdate } = mockTabManager({
        browser,
        initialTabs,
        initialWindows,
        filterStorage,
      });
      await expectTab(nextUpdate(), 21);
      changeTab(21, tab => (tab.url = 'https://github.com/notification'));
      await expectNothing(nextUpdate());
      setMode(FilterMode.Allow);
      await expectTab(nextUpdate(), 21);
      setMode(FilterMode.Block);
      await expectNothing(nextUpdate());
      setFilters([]);
      await expectTab(nextUpdate(), 21);
    });
    it('should keep track of filtered tabs', async function () {
      jest.setTimeout(99999999);
      const { browser, initialTabs, initialWindows, changeTab, addTab, focusWindow, removeTab } =
        make1Wind1TabBrowser();
      const { setMode, filterStorage } = dynamicStorage([{ value: 'github.com', isRegex: false }], FilterMode.Block);
      const { nextUpdate } = mockTabManager({
        browser,
        initialTabs,
        initialWindows,
        filterStorage,
      });
      await expectNothing(nextUpdate());
      addTab(mockAudibleTab(21, 2, { title: 'aliens!', url: 'https://github.com/notifications', active: true }));
      await expectTab(nextUpdate(), 11);
      focusWindow(1);
      await expectNothing(nextUpdate());
      changeTab(21, tab => (tab.url = 'https://gitlab.com'));
      await expectTab(nextUpdate(), 21);
      removeTab(21);
      await expectNothing(nextUpdate());
      const promise = nextUpdate();
      setMode(FilterMode.Allow);
      addTab(mockAudibleTab(22, 2, { title: 'aliens2!', url: 'https://github.com/notifications', active: true }));
      // "flush" the remaining actions
      await waitNMicrotasks(5);
      focusWindow(1);
      await expectTab(promise, 22);
      setMode(FilterMode.Block);
      await expectNothing(nextUpdate());
    });
  });

  describe('overridden-sessions', function () {
    it('should update tabs if they get metadata when active', async function () {
      const { browser, initialTabs, initialWindows, addTab, focusWindow } = make1Wind1TabBrowser();
      const { nextUpdate, manager } = mockTabManager({
        browser,
        initialTabs,
        initialWindows,
      });
      const targetTab = mockAudibleTab(21, 2, { title: 'MediaSession - xd', active: true });
      addTab(targetTab);
      await waitNMicrotasks(5);
      focusWindow(1);
      await expectTab(nextUpdate(), 21);

      let promise = nextUpdate();
      manager.setMetadata(targetTab, { title: 'aliens', artist: 'pleased' });
      let emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.title).toBe('aliens');
      expect(emitted.artist).toBe('pleased');
      expect(emitted.timeline).toBeNull();

      promise = nextUpdate();
      manager.setPlayPosition(targetTab, { position: 1, duration: 2, rate: 1, timestamp: Date.now() });
      emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.timeline).not.toBeNull();

      promise = nextUpdate();
      manager.setMetadata(targetTab);
      emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.title).not.toBe('aliens');
      expect(emitted.artist).not.toBe('pleased');
      expect(emitted.timeline).not.toBeNull();

      promise = nextUpdate();
      manager.setPlayPosition(targetTab);
      emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.title).not.toBe('aliens');
      expect(emitted.artist).not.toBe('pleased');
      expect(emitted.timeline).toBeNull();
    });

    it('should update tabs if their metadata updates when not active', async function() {
      const { browser, initialTabs, initialWindows, addTab, focusWindow } = make1Wind1TabBrowser();
      const { nextUpdate, manager } = mockTabManager({
        browser,
        initialTabs,
        initialWindows,
      });
      await expectNothing(nextUpdate());
      const targetTab = mockAudibleTab(21, 2, { title: 'MediaSession - xd', active: true });
      addTab(targetTab);
      await expectTab(nextUpdate(), 11);

      let promise = nextUpdate();
      manager.setMetadata(targetTab, { title: 'aliens', artist: 'pleased' });
      focusWindow(1);
      let emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.title).toBe('aliens');
      expect(emitted.artist).toBe('pleased');
      expect(emitted.timeline).toBeNull();

      focusWindow(2);
      await expectTab(nextUpdate(), 11);
      promise = nextUpdate();
      manager.setPlayPosition(targetTab, { position: 1, duration: 2, rate: 1, timestamp: Date.now() });
      focusWindow(1);
      emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.timeline).not.toBeNull();

      focusWindow(2);
      await expectTab(nextUpdate(), 11);
      promise = nextUpdate();
      manager.setMetadata(targetTab);
      focusWindow(1);
      emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.title).not.toBe('aliens');
      expect(emitted.artist).not.toBe('pleased');
      expect(emitted.timeline).not.toBeNull();

      focusWindow(2);
      await expectTab(nextUpdate(), 11);
      promise = nextUpdate();
      manager.setPlayPosition(targetTab);
      focusWindow(1);
      emitted = (await expectTab(promise, 21)).createPlayInfo();
      expect(emitted.title).not.toBe('aliens');
      expect(emitted.artist).not.toBe('pleased');
      expect(emitted.timeline).toBeNull();
    });
  });
});
