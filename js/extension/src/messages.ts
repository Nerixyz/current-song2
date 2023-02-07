import { VideoMetadata, VideoPlayMode, VideoPlayPosition } from './types/video.types';

export type InternalMessageMap = {
  PlayPosition: VideoPlayPosition | null;
  Metadata: VideoMetadata | null;
  PlayMode: VideoPlayMode;
};
