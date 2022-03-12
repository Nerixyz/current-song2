import { FilterModel } from './types';
import { DEFAULT_FILTER_MODE, FilterMode } from '../options';
import { IFilterStorage, RevokeListener } from './FilterStorage';

export class FilterManager {
  private filtersUpdated?: () => void;
  private readonly listenFiltersHandle: RevokeListener;
  private readonly listenFilterModeHandle: RevokeListener;

  private filters: FilterModel[] = [];
  private filterMode: FilterMode = DEFAULT_FILTER_MODE;

  constructor(storage: IFilterStorage, updateListener?: () => void) {
    this.filtersUpdated = updateListener;
    this.listenFiltersHandle = storage.listenFilters(f => this.updateFilters(f));
    this.listenFilterModeHandle = storage.listenFilterMode(m => this.updateFilterMode(m));
  }

  public setUpdateListener(listener: () => void) {
    this.filtersUpdated = listener;
  }

  public checkUrl(target: string): boolean {
    const url = tryParseUrl(target);
    if (!url)
      // If the mode is to only block urls, then this url won't be blocked
      return this.filterMode === FilterMode.Block;

    const anyMatch = this.filters.some(model => matchModel(url, model));
    // If the mode is 'block' then if there is any match, the url is not "valid".
    return this.filterMode === FilterMode.Block ? !anyMatch : anyMatch;
  }

  public close() {
    this.listenFiltersHandle();
    this.listenFilterModeHandle();
  }

  private updateFilters(filters?: FilterModel[]) {
    this.filters = filters ?? this.filters;
    if (filters) this.filtersUpdated?.();
  }

  private updateFilterMode(mode?: FilterMode) {
    const shouldUpdate = mode && this.filterMode !== mode;
    this.filterMode = mode ?? this.filterMode;
    if (shouldUpdate) this.filtersUpdated?.();
  }
}

export function tryParseUrl(url: string): URL | null {
  try {
    return new URL(url);
  } catch {
    if (!url.startsWith('http')) return tryParseUrl('https://' + url);
    return null;
  }
}

export function matchModel(url: URL, model: FilterModel): boolean {
  if (model.isRegex) {
    return new RegExp(model.value).test(url.href);
  }
  return url.hostname.startsWith(model.value) || url.hostname.endsWith(model.value);
}
