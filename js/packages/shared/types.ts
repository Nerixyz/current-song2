export interface PlayInfo {
  title: string;
  artist: string;
  image: null | ImageInfo;
  timeline: null | TimelineInfo;

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
