export function splitTitle(title: string): { title: string; artist?: string } {
  if (title.includes('-') && !title.match(/\([^()]+-[^()]+\)/)) {
    const [first, ...second] = title.split('-');
    return { artist: first, title: second.join(' ') };
  } else if (title.includes('by')) {
    // used by SoundCloud
    const [first, ...second] = title.split('by');
    return { artist: second.join(' ').trim(), title: first.trim() };
  } else {
    return { title: title };
  }
}
