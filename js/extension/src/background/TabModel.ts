import { BrowserTab, TabId, WindowId } from '../types/tab.types';
import { VideoMetadata, VideoPlayPosition } from '../types/video.types';
import { PlayInfo, TimelineInfo } from '../../../shared/types';
import { LegacyEventData, MessageCreator } from '../types/message.types';
import { splitTitle } from '../utils/text';

export enum TabChange {
  NotChanged = 0x0,
  UrlChanged = 0x1,
  MetaChanged = 0x2,
}

// TODO: check chrome
const DEFAULT_URL = 'about:blank';

export class TabModel implements MessageCreator {
  id: TabId;
  windowId: WindowId = -1;
  active = false;
  audible = false;
  muted = false;

  private title = '';
  private artist: string | null = null;
  private albumName: string | null = null;

  private hasMetadata = false;
  private tabTitle = '';

  private timeline: TimelineInfo | null = null;
  private imageUrl: string | null = null;

  private _url = DEFAULT_URL;
  get url() {
    return this._url;
  }

  /** @throws if `tab.id` is `undefined`
   */
  constructor(tab: BrowserTab) {
    if (typeof tab.id !== 'number') throw new Error('Cannot create tab without id');
    this.id = tab.id;
    this.updateTabMeta(tab);
  }

  createLegacyEvent(): LegacyEventData {
    return {
      metadata: {
        title: this.title,
        artist: this.artist ?? undefined,
        artwork: this.imageUrl ?? undefined,
      },
      position: this.timeline
        ? {
            rate: this.timeline.rate,
            timestamp: this.timeline.ts,
            position: this.timeline.progressMs / 1000,
            duration: this.timeline.durationMs / 1000,
          }
        : undefined,
    };
  }
  createPlayInfo(): PlayInfo {
    return {
      title: this.title,
      artist: this.artist ?? '',
      source: 'browser',
      image: this.imageUrl,
      timeline: this.timeline,
      album: this.albumName
        ? {
            title: this.albumName,
            trackCount: 0,
          }
        : null,
      trackNumber: null,
    };
  }

  /**
   * @param {BrowserTab} tab
   * @returns {boolean} true if the tab has changed. This could include
   *  1) change in active state
   *  2) change in audible state
   *  3) change in windowId
   *  4) change in title/artist
   */
  updateTabMeta(tab: BrowserTab): TabChange {
    const isEqual =
      this.windowId === (tab.windowId ?? this.windowId) &&
      this.active === tab.active &&
      this.audible === !!tab.audible &&
      this.muted === (tab.mutedInfo?.muted ?? this.muted) &&
      (this.hasMetadata || this.tabTitle === (tab.title ?? ''));
    const urlEqual = (tab.url ?? this._url) === this._url;

    this.tabTitle = tab.title ?? '';
    this.windowId = tab.windowId ?? this.windowId;
    this.active = tab.active;
    this.audible = !!tab.audible;
    this.muted = tab.mutedInfo?.muted ?? this.muted;
    this._url = tab.url ?? this._url;

    if (!this.hasMetadata) this.tryExtractSetArtistFromTitle();

    return isEqual ? (urlEqual ? TabChange.NotChanged : TabChange.UrlChanged) : TabChange.MetaChanged;
  }

  setActive(active: boolean) {
    this.active = active;
  }

  updateMetadata(meta?: VideoMetadata): boolean {
    if (meta) {
      this.hasMetadata = true;

      const imageChange = this.imageUrl !== (meta.artwork || null);
      this.imageUrl = meta.artwork || null;

      const albumChange = this.albumName !== (meta.album || null);
      this.albumName = meta.album || null;

      if (meta.artist) {
        if (meta.title.includes('-')) {
          // a workaround for YouTube
          const { title, artist } = splitTitle(meta.title);
          meta.title = title.trim();
          meta.artist = artist?.trim() ?? meta.artist;
        }

        const anyChange = albumChange || imageChange || this.title !== meta.title || this.artist !== meta.artist;

        this.title = meta.title;
        this.artist = meta.artist;

        return anyChange;
      } else {
        return this.tryExtractSetArtist(meta.title) || imageChange || albumChange;
      }
    } else {
      this.hasMetadata = false;
      this.imageUrl = null;
      const albumChange = !!this.albumName;
      this.albumName = null;
      const metaChange = this.tryExtractSetArtistFromTitle();
      return metaChange || albumChange;
    }
  }

  updateTimeline(position?: VideoPlayPosition) {
    this.timeline = position
      ? {
          ts: Math.round(position.timestamp),
          rate: position.rate,
          progressMs: Math.round(position.position),
          durationMs: Math.round(position.duration),
        }
      : null;
  }

  private tryExtractSetArtistFromTitle(): boolean {
    return this.tryExtractSetArtist(this.tabTitle.replace(/ +- [^-]+$/, ''));
  }

  private tryExtractSetArtist(fullTitle: string): boolean {
    const { title, artist } = splitTitle(fullTitle);
    const anyChange = this.title !== title || this.artist !== (artist || null);
    this.title = title;
    // use || instead of ??, so that '' will be null
    this.artist = artist || null;

    return anyChange;
  }
}
