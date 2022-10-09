import { PlayInfo } from '../../shared/types';

export type EventMap = {
  Playing: PlayInfo;
  Paused: undefined | null;
};

export * from '../../shared/types';
