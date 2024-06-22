import { formatLocalUrl } from '../../shared/url';
import { PlayInfo } from '../../shared/types';

export function getImageUrl(info: PlayInfo): string | undefined {
  if (!info.image) return undefined;

  if (typeof info.image === 'string') {
    if (!info.image.startsWith('https://')) return undefined;
    return info.image;
  } else {
    return formatLocalUrl({
      path: `/api/img/${info.image.id}/${info.image.epochId}`,
      port: Number(location.port),
      host: location.hostname,
    });
  }
}
