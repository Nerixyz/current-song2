import { PlayInfo } from './types';
import { cleanupTitleAndSub, extractTitleAndSub } from './text';
import { getImageUrl } from './image';

export interface State {
  info: PlayInfo;
  title: string;
  subtitle: string | undefined;
  imageUrl: string | undefined;
}

export function makeState(info: PlayInfo): State {
  const extracted = extractTitleAndSub(info);
  const { title, subtitle } = cleanupTitleAndSub(extracted);
  const imageUrl = getImageUrl(info);
  return {
    info,
    title,
    subtitle,
    imageUrl,
  };
}

export function isSpotify(state: State): boolean {
  return (
    state.info.source.startsWith('gsmtc') && state.info.source.toLowerCase().includes('spotify')
  );
}

export function hasImage(state: State) {
  return !!state.imageUrl;
}

export function hasSubtitle(state: State) {
  return !!state.subtitle;
}

export function hasTimeline(state: State) {
  return !!state.info.timeline;
}

export function not<T>(fn: (state: T) => boolean): (state: T) => boolean {
  return s => !fn(s);
}
