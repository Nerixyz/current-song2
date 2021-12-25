import { BrowserTab, TabId, WindowId } from '../types/tab.types';
import { VideoMetadata, VideoPlayPosition } from '../types/video.types';
import { PlayInfo, TimelineInfo } from '../../../shared/types';
import { LegacyEventData, MessageCreator } from '../types/message.types';
import { splitTitle } from '../utils/text';

export class TabModel implements MessageCreator {
  id: TabId = -1;
  windowId: WindowId = -1;
  active = false;
  audible = false;
  muted = false;

  private title = '';
  private artist: string | null = null;

  private hasMetadata = false;
  private tabTitle = '';

  private timeline: TimelineInfo | null = null;
  private imageUrl: string | null = null;

  constructor(tab: BrowserTab) {
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
  updateTabMeta(tab: BrowserTab): boolean {
    const isEqual =
      this.windowId === tab.windowId &&
      this.active === tab.active &&
      this.audible === !!tab.audible &&
      this.muted === (tab.mutedInfo?.muted ?? this.muted) &&
      (this.hasMetadata || this.tabTitle === (tab.title ?? ''));

    this.tabTitle = tab.title ?? '';
    this.windowId = tab.windowId ?? -1;
    this.id = tab.id ?? -1;
    this.active = tab.active;
    this.audible = !!tab.audible;
    this.muted = tab.mutedInfo?.muted ?? this.muted;

    if (!this.hasMetadata) this.tryExtractSetArtistFromTitle();

    return !isEqual;
  }

  setActive(active: boolean) {
    this.active = active;
  }

  updateMetadata(meta?: VideoMetadata): boolean {
    if (meta) {
      this.hasMetadata = true;
      const imageChange = this.imageUrl !== (meta.artwork || null);
      this.imageUrl = meta.artwork || null;
      if (meta.artist) {
        if (meta.title.includes('-')) {
          // a workaround for YouTube
          const { title, artist } = splitTitle(meta.title);
          meta.title = title.trim();
          meta.artist = artist?.trim() ?? meta.artist;
        }

        const anyChange = imageChange || this.title !== meta.title || this.artist !== meta.artist;

        this.title = meta.title;
        this.artist = meta.artist;

        return anyChange;
      } else {
        return this.tryExtractSetArtist(meta.title) || imageChange;
      }
    } else {
      this.hasMetadata = false;
      this.imageUrl = null;
      return this.tryExtractSetArtistFromTitle();
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
    this.title = title.trim();
    // use || instead of ??, so that '' will be null
    this.artist = artist?.trim() || null;

    return anyChange;
  }
}
