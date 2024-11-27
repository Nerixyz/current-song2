import { cleanupTitleAndSub, extractTitleAndSub } from './text';
import { getImageUrl } from './image';
import { PlayInfo } from '../../shared/types';
import { ScriptOptions } from './options';

export interface State {
  info: PlayInfo;
  title: string;
  subtitle: string | undefined;
  imageUrl: string | undefined;
}

export function makeState(info: PlayInfo, options: ScriptOptions): State {
  let extracted = extractTitleAndSub(info);
  if (!options.useRawSongInfo) {
    extracted = cleanupTitleAndSub(extracted);
  }
  const { title, subtitle } = extracted;

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

export function hasAlbum(state: State) {
  return !!state.info.album;
}

export function hasValidAlbumTracks(state: State) {
  return (state.info.album?.trackCount ?? 0) > 0;
}

export function hasTrack(state: State) {
  return (state.info.trackNumber ?? 0) > 0;
}

export function not<T>(fn: (state: T) => boolean): (state: T) => boolean {
  return s => !fn(s);
}
