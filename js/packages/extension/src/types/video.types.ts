export interface VideoPlayPosition {
  rate: number;
  timestamp: number;
  duration: number;
  position: number;
}

export type VideoPlayMode = 'playing' | 'paused' | 'none';

export interface VideoMetadata {
  title: string;
  artist?: string;
  artwork?: string;
  album?: string;
}
