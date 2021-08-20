import { PlayInfo } from './types';
import { formatLocalUrl } from './url';
import { State } from './state';

export function getImageUrl(info: PlayInfo): string | undefined {
  if (!info.image) return undefined;

  if (typeof info.image === 'string') {
    if (!info.image.startsWith('https://')) return undefined;
    return info.image;
  } else {
    return formatLocalUrl(`/api/img/${info.image.id}/${info.image.epochId}`);
  }
}
