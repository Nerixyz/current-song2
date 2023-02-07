import { VideoMetadata, VideoPlayPosition } from './video.types';
import { PlayInfo } from '../../../shared/types';

export interface MessageCreator {
  createLegacyEvent(): LegacyEventData;
  createPlayInfo(): PlayInfo;
}

export interface LegacyEventData {
  metadata: VideoMetadata;
  position?: VideoPlayPosition;
}
