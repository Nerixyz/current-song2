import { BrowserTab } from '../../src/types/tab.types';
import { TabModel } from '../../src/background/TabModel';
import { TimelineInfo } from '../../../shared/types';

export function mockTab(
  id: number,
  windowId: number,
  other: Partial<Exclude<BrowserTab, 'id' | 'windowId'>> = {},
): BrowserTab {
  return {
    id,
    windowId,
    active: false,
    pinned: false,
    highlighted: false,
    incognito: false,
    index: 0,
    ...other,
  };
}

export function mockAudibleTab(
  id: number,
  windowId: number,
  other: Partial<Exclude<BrowserTab, 'id' | 'windowId' | 'audible'>> = {},
): BrowserTab {
  return mockTab(id, windowId, { ...other, audible: true });
}

export function mockAudibleMutedTab(
  id: number,
  windowId: number,
  other: Partial<Exclude<BrowserTab, 'id' | 'windowId' | 'audible' | 'mutedInfo'>> = {},
): BrowserTab {
  return mockTab(id, windowId, { ...other, audible: true, mutedInfo: { muted: true } });
}

export function mockMutedTab(
  id: number,
  windowId: number,
  other: Partial<Exclude<BrowserTab, 'id' | 'windowId' | 'mutedInfo'>> = {},
): BrowserTab {
  return mockTab(id, windowId, { ...other, mutedInfo: { muted: true } });
}

interface TabStateProps {
  id: number;
  windowId: number;
  active: boolean;
  audible: boolean;
  muted: boolean;
}
export function expectTabState(tab: TabModel, props: TabStateProps) {
  expect<TabStateProps>({
    id: tab.id,
    windowId: tab.windowId,
    active: tab.active,
    audible: tab.audible,
    muted: tab.muted,
  }).toEqual(props);
}

export function expectTitleArtist(tab: TabModel, expTitle: string, expArtist: string, expImageUrl?: string | null) {
  const { title, artist, image } = tab.createPlayInfo();

  expect(title).toBe(expTitle);
  expect(artist).toBe(expArtist);
  if (typeof expImageUrl !== 'undefined') {
    expect(image).toBe(expImageUrl);
  }
}

export function expectTimeline(tab: TabModel, tl: TimelineInfo | null) {
  const { timeline } = tab.createPlayInfo();
  expect(timeline).toEqual(tl);
}
