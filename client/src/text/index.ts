import { PlayInfo } from '../types';

const MAX_SAFE_STRING = 50;

interface TitleData {
  title: string;
  subtitle?: string;
}

export function extractTitleAndSub(info: PlayInfo): TitleData {
  if (info.artist) {
    return { title: info.title, subtitle: info.artist };
  }

  let title = info.title;
  if (info.source.startsWith('gsmtc') && info.source.match(/chrome|firefox|opera|brave|edge/i)) {
    console.log('xd');
    const dashIdx = title.lastIndexOf('-');
    if (dashIdx !== -1) {
      title = title.substring(0, dashIdx).trim();
    }
  }

  const dashIdx = title.indexOf('-');

  return dashIdx === -1
    ? { title }
    : {
        title: title.substring(dashIdx + 1).trim(),
        subtitle: title.substring(0, dashIdx).trim(),
      };
}

export function cleanupTitleAndSub(data: TitleData): TitleData {
  if (data.title.length > MAX_SAFE_STRING) {
    let title = cleanupTooLongString(data.title);
    let subtitle = data.subtitle;
    if (title.includes('-')) {
      const dashIdx = title.indexOf('-');
      subtitle = title.substring(0, dashIdx).trim();
      title = title.substring(dashIdx + 1).trim();
    }
    if (subtitle && subtitle?.length > MAX_SAFE_STRING) {
      subtitle = cleanupTooLongString(subtitle);
    }

    return { title, subtitle };
  } else if (data.subtitle && data.subtitle.length > MAX_SAFE_STRING) {
    return { title: data.title, subtitle: cleanupTooLongString(data.subtitle) };
  }
  return data;
}

function cleanupTooLongString(str: string): string {
  return str.replace(/(:?\(.+\)|\[.+]|{.+})/g, '').replace(/ +/, ' ').trim();
}
