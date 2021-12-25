import { TabModel } from '../../src/background/TabModel';
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
  });
  describe('updateTabMeta', function () {
    it('should return false if nothing changed', function () {
      const model1 = new TabModel(mockTab(1, 2));
      expect(model1.updateTabMeta(mockTab(1, 2))).toBe(false);
      const model2 = new TabModel(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Dancing - YouTube' }));
      expect(
        model2.updateTabMeta(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Dancing - YouTube' })),
      ).toBe(false);
      const model3 = new TabModel(mockTab(1, 2, { audible: true }));
      expect(model3.updateTabMeta(mockTab(1, 2, { audible: true }))).toBe(false);
      const model4 = new TabModel(mockTab(1, 2, { active: true }));
      expect(model4.updateTabMeta(mockTab(1, 2, { active: true }))).toBe(false);

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
      expect(model1.updateTabMeta(mockTab(1, 2, { mutedInfo: { muted: true } }))).toBe(true);
      const model2 = new TabModel(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Dancing - YouTube' }));
      expect(
        model2.updateTabMeta(mockTab(1, 2, { mutedInfo: { muted: true }, title: 'Alien - Danced - YouTube' })),
      ).toBe(true);
      const model3 = new TabModel(mockTab(1, 2, { audible: true }));
      expect(model3.updateTabMeta(mockTab(1, 2, { audible: false }))).toBe(true);
      const model4 = new TabModel(mockTab(1, 2, { active: true }));
      expect(model4.updateTabMeta(mockTab(1, 2, { active: false, audible: true }))).toBe(true);

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
      expect(model.updateTabMeta(mockTab(1, 2, { title: 'Opps - Dank - YouTube' }))).toBe(false);
      expect(model.updateMetadata({ title: 'Imagination', artist: 'Not me' })).toBe(false);
      expectTitleArtist(model, 'Imagination', 'Not me');
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
      expectTitleArtist(model, '', '', null);
      expect(model.updateMetadata({ title: 'Dancing', artist: 'Alien', artwork: 'https://nerixyz.de' })).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de');
    });
    it('should split the title if it contains a dash', function () {
      const model = new TabModel(mockTab(1, 2));
      expectTitleArtist(model, '', '', null);
      expect(
        model.updateMetadata({ title: 'Alien - Dancing', artist: 'Some Uploader', artwork: 'https://nerixyz.de' }),
      ).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de');
    });
    it('should return false if nothing got updated', function () {
      const model = new TabModel(mockTab(1, 2));
      expectTitleArtist(model, '', '', null);
      expect(
        model.updateMetadata({ title: 'Alien - Dancing', artist: 'Some Uploader', artwork: 'https://nerixyz.de' }),
      ).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de');
      expect(
        model.updateMetadata({ title: 'Alien - Dancing', artist: 'Some Uploader', artwork: 'https://nerixyz.de' }),
      ).toBe(false);
      expectTitleArtist(model, 'Dancing', 'Alien', 'https://nerixyz.de');
    });
    it('should return false if the title was set by the tab', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Alien - Dancing - YouTube' }));
      expectTitleArtist(model, 'Dancing', 'Alien', null);
      expect(model.updateMetadata({ title: 'Alien - Dancing', artist: 'Some Uploader' })).toBe(false);
      expectTitleArtist(model, 'Dancing', 'Alien', null);
    });

    it('should fall back to the tab title if no metadata is given', function () {
      const model = new TabModel(mockTab(1, 2, { title: 'Alien - Dancing - YouTube' }));
      expectTitleArtist(model, 'Dancing', 'Alien', null);
      expect(model.updateMetadata({ title: 'Me - Danced', artist: 'Some Uploader' })).toBe(true);
      expectTitleArtist(model, 'Danced', 'Me', null);
      expect(model.updateMetadata()).toBe(true);
      expectTitleArtist(model, 'Dancing', 'Alien', null);
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
});
