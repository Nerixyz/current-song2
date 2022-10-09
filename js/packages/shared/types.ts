export interface PlayInfo {
  title: string;
  artist: string;
  trackNumber: null | number;

  image: null | ImageInfo;
  timeline: null | TimelineInfo;
  album: null | AlbumInfo;

  source: string;
}

export type ImageInfo = string | InternalImage;

export interface InternalImage {
  id: number;
  epochId: number;
}

export interface TimelineInfo {
  ts: number;
  durationMs: number;
  progressMs: number;
  rate: number;
}

export interface AlbumInfo {
  title: string,
  trackCount: number,
}
