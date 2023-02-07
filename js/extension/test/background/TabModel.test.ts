import { TabChange, TabModel } from '../../src/background/TabModel';
import { expectTabState, expectTimeline, expectTitleArtist, mockTab } from '../mock/tab';

describe('TabModel', function () {
  describe('constructor', function () {
    it('should initialize correctly', function () {
      expectTabState(new TabModel(mockTab(1, 2)), {
        id: 1,
        windowId: 2,
        muted: false,
        audible: false,
        active: false,
      });
      expectTabState(new TabModel(mockTab(1, 2, { mutedInfo: { muted: true } })), {
        id: 1,
        windowId: 2,
        muted: true,
        audible: false,
        active: false,
      });
      expectTabState(new TabModel(mockTab(1, 2, { audible: true })), {
        id: 1,
        windowId: 2,
        muted: false,
        audible: true,
        active: false,
      });
      expectTabState(new TabModel(mockTab(1, 2, { active: true })), {
        id: 1,
        windowId: 2,
        muted: false,
        audible: false,
        active: true,
      });
    });
    it('should throw if the tab does not have an id', function () {
      const tab = mockTab(1, 2);
      tab.id = undefined;
      expect(() => new TabModel(tab)).toThrow();
    });
  });

  describe('#updateTabMeta', function () {
    it('should return false if nothing changed', function () {
      const model1 = new TabModel(mockTab(1, 2));
      expect(model1.updateTabMeta(mockTab(1, 2))).toBe(TabChange.NotChanged);
      const model2 = new TabModel(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Dancing - YouTube' }));
      expect(
        model2.updateTabMeta(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Dancing - YouTube' })),
      ).toBe(TabChange.NotChanged);
      const model3 = new TabModel(mockTab(1, 2, { audible: true }));
      expect(model3.updateTabMeta(mockTab(1, 2, { audible: true }))).toBe(TabChange.NotChanged);
      const model4 = new TabModel(mockTab(1, 2, { active: true }));
      expect(model4.updateTabMeta(mockTab(1, 2, { active: true }))).toBe(TabChange.NotChanged);

      expectTabState(model1, {
        id: 1,
        windowId: 2,
        muted: false,
        audible: false,
        active: false,
      });
      expectTabState(model2, {
        id: 1,
        windowId: 2,
        muted: true,
        audible: false,
        active: false,
      });
      expectTitleArtist(model2, 'Dancing', 'Alien');
      expectTabState(model3, {
        id: 1,
        windowId: 2,
        muted: false,
        audible: true,
        active: false,
      });
      expectTabState(model4, {
        id: 1,
        windowId: 2,
        muted: false,
        audible: false,
        active: true,
      });
    });

    it('should return true if something is updated', function () {
      const model1 = new TabModel(mockTab(1, 2));
      expect(model1.updateTabMeta(mockTab(1, 2, { mutedInfo: { muted: true } }))).toBe(TabChange.MetaChanged);
      const model2 = new TabModel(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Dancing - YouTube' }));
      expect(
        model2.updateTabMeta(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Danced - YouTube' })),
      ).toBe(TabChange.MetaChanged);
      const model3 = new TabModel(mockTab(1, 2, { audible: true }));
      expect(model3.updateTabMeta(mockTab(1, 2, { audible: false }))).toBe(TabChange.MetaChanged);
      const model4 = new TabModel(mockTab(1, 2, { active: true }));
      expect(model4.updateTabMeta(mockTab(1, 2, { active: false, audible: true }))).toBe(TabChange.MetaChanged);

      expectTabState(model1, {
        id: 1,
        windowId: 2,
        muted: true,
        audible: false,
        active: false,
      });
      expectTabState(model2, {
        id: 1,
        windowId: 2,
        muted: true,
        audible: false,
        active: false,
      });
      expectTitleArtist(model2, 'Danced', 'Alien');
      expectTabState(model3, {
        id: 1,
        windowId: 2,
        muted: false,
        audible: false,
        active: false,
      });
      expectTabState(model4, {
        id: 1,
        windowId: 2,
        muted: false,
        audible: true,
        active: false,
      });
    });

    it('should return false if the title is updated but metadata exists', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Alien - Dancing - YouTube' }));
      expectTitleArtist(model, 'Dancing', 'Alien');
      expect(model.updateMetadata({ title: 'Imagination', artist: 'Not me' })).toBe(true);
      expectTitleArtist(model, 'Imagination', 'Not me');
      expect(model.updateTabMeta(mockTab(1, 2, { title: 'Opps - Dank - YouTube' }))).toBe(TabChange.NotChanged);
      expect(model.updateMetadata({ title: 'Imagination', artist: 'Not me' })).toBe(false);
      expectTitleArtist(model, 'Imagination', 'Not me');
    });

    it('should update the url correctly', function () {
      const model = new TabModel(mockTab(1, 2, { url: 'http://localhost', title: 'Aliens - Dancing - YouTube' }));
      expect(model.url).toBe('http://localhost');
      expect(
        model.updateTabMeta(mockTab(1, 2, { url: 'http://localhost:8080', title: 'Aliens - Dancing - YouTube' })),
      ).toBe(TabChange.UrlChanged);
      expect(model.url).toBe('http://localhost:8080');
    });
    it('should handle tabs without a windowId', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Aliens - Dancing - YouTube' }));
      expect(model.id).toBe(1);
      expect(model.windowId).toBe(2);
      const tab = mockTab(1, 2, { title: 'Aliens - Dancing - YouTube' });
      tab.windowId = undefined;
      expect(model.updateTabMeta(tab)).toBe(TabChange.NotChanged);
      expect(model.windowId).toBe(2);
    });
  });

  describe('#setActive', function () {
    it('should set the active state', function () {
      const model = new TabModel(mockTab(1, 2));
      expect(model.active).toBe(false);
      model.setActive(true);
      expect(model.active).toBe(true);
      model.setActive(false);
      expect(model.active).toBe(false);
    });
  });

  describe('#updateMetadata', function () {
    it('should set the title and artist', function () {
      const model = new TabModel(mockTab(1, 2));
      expectTitleArtist(model, '', '', null, null);
      expect(
        model.updateMetadata({ title: 'Dancing', artist: 'Alien', artwork: 'https://nerixyz.de', album: 'wow' }),
      ).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de', 'wow');
    });
    it('should split the title if it contains a dash', function () {
      const model = new TabModel(mockTab(1, 2));
      expectTitleArtist(model, '', '', null, null);
      expect(
        model.updateMetadata({
          title: 'Alien - Dancing',
          artist: 'Some Uploader',
          artwork: 'https://nerixyz.de',
          album: 'wow',
        }),
      ).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de', 'wow');
    });
    it('should return false if nothing got updated', function () {
      const model = new TabModel(mockTab(1, 2));
      expectTitleArtist(model, '', '', null, null);
      expect(
        model.updateMetadata({
          title: 'Alien - Dancing',
          artist: 'Some Uploader',
          artwork: 'https://nerixyz.de',
          album: 'wow',
        }),
      ).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de', 'wow');
      expect(
        model.updateMetadata({
          title: 'Alien - Dancing',
          artist: 'Some Uploader',
          artwork: 'https://nerixyz.de',
          album: 'wow',
        }),
      ).toBe(false);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de', 'wow');
    });
    it('should return false if the title was set by the tab', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Alien - Dancing - YouTube' }));
      expectTitleArtist(model, 'Dancing', 'Alien', null, null);
      expect(model.updateMetadata({ title: 'Alien - Dancing', artist: 'Some Uploader' })).toBe(false);
      expectTitleArtist(model, 'Dancing', 'Alien', null, null);
    });
    it('should fall back to the tab title if no metadata is given', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Alien - Dancing - YouTube' }));
      expectTitleArtist(model, 'Dancing', 'Alien', null, null);
      expect(model.updateMetadata({ title: 'Me - Danced', artist: 'Some Uploader' })).toBe(true);
      expectTitleArtist(model, 'Danced', 'Me', null, null);
      expect(model.updateMetadata()).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', null, null);
      expect(model.updateMetadata({ title: 'I - Failed' })).toBe(true);
      expectTitleArtist(model, 'Failed', 'I', null, null);
      expect(model.updateMetadata({ title: 'I - Failed', artwork: 'aliens.jxl' })).toBe(true);
      expectTitleArtist(model, 'Failed', 'I', 'aliens.jxl', null);
      expect(model.updateMetadata({ title: 'I - Failed', artwork: 'aliens.jxl', album: 'wow' })).toBe(true);
      expectTitleArtist(model, 'Failed', 'I', 'aliens.jxl', 'wow');
      expect(model.updateMetadata({ title: 'I - Failed', artwork: 'aliens.jxl', album: 'wow' })).toBe(false);
    });
    it('should fall back to the metadata artist if none is in the title', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Alien - Dancing - YouTube' }));
      expectTitleArtist(model, 'Dancing', 'Alien', null, null);
      expect(model.updateMetadata({ title: 'Aliens (Forsen-Remix)', artist: 'Forsen' })).toBe(true);
      expectTitleArtist(model, 'Aliens (Forsen-Remix)', 'Forsen', null, null);
    });
  });

  describe('#updateTimeline', function () {
    it('should set the timeline', function () {
      const model = new TabModel(mockTab(1, 2));
      expectTimeline(model, null);
      model.updateTimeline({ timestamp: 12.1, rate: 1.3, position: 13.4, duration: 1.4 });
      expectTimeline(model, { ts: 12, rate: 1.3, durationMs: 1, progressMs: 13 });
    });
  });

  describe('#creteLegacyEvent', function () {
    it('should return the correct metadata', function () {
      const model = new TabModel(mockTab(1, 2));
      expect(model.createLegacyEvent()).toStrictEqual({
        metadata: { title: '', artist: undefined, artwork: undefined },
        position: undefined,
      });
      model.updateMetadata({ title: 'Forsen', artist: 'Alien' });
      expect(model.createLegacyEvent()).toStrictEqual({
        metadata: { title: 'Forsen', artist: 'Alien', artwork: undefined },
        position: undefined,
      });
      model.updateMetadata({ title: 'Forsen', artist: 'Alien', artwork: 'xd.jxl' });
      expect(model.createLegacyEvent()).toStrictEqual({
        metadata: { title: 'Forsen', artist: 'Alien', artwork: 'xd.jxl' },
        position: undefined,
      });
      model.updateTimeline({ timestamp: 12345, rate: 1.5, position: 2000, duration: 4000 });
      expect(model.createLegacyEvent()).toStrictEqual({
        metadata: { title: 'Forsen', artist: 'Alien', artwork: 'xd.jxl' },
        position: { timestamp: 12345, rate: 1.5, position: 2, duration: 4 },
      });
      model.updateMetadata();
      expect(model.createLegacyEvent()).toStrictEqual({
        metadata: { title: '', artist: undefined, artwork: undefined },
        position: { timestamp: 12345, rate: 1.5, position: 2, duration: 4 },
      });
    });
  });
});
