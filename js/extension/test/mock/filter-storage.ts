import { FilterModel } from '../../src/filters/types';
import { FilterMode } from '../../src/options';
import { IFilterStorage } from '../../src/filters/FilterStorage';

export const staticStorage = (
  filters: FilterModel[],
  mode: FilterMode,
  includeFocusedTabs: boolean,
): IFilterStorage => ({
  listenFilterMode(cb) {
    cb(mode);
    return () => undefined;
  },
  listenFilters(cb) {
    cb(filters);
    return () => undefined;
  },
  listenIncludeFocusedTabs(cb) {
    cb(includeFocusedTabs);
    return () => undefined;
  },
});

export const dynamicStorage = (
  initialFilters: FilterModel[],
  initialMode: FilterMode,
  initialIncludeFocusedTabs: boolean,
) => {
  let filterModeCb: null | ((mode: FilterMode) => void) = null;
  let filtersCb: null | ((filters: FilterModel[]) => void) = null;
  let includeFocusedTabsCb: null | ((includeFocusedTabs: boolean) => void) = null;
  return {
    setFilters: (filters: FilterModel[]) => queueMicrotask(() => filtersCb?.(filters)),
    setMode: (mode: FilterMode) => queueMicrotask(() => filterModeCb?.(mode)),
    setIncludeFocusedTabs: (includeFocusedTabs: boolean) =>
      queueMicrotask(() => includeFocusedTabsCb?.(includeFocusedTabs)),
    filterStorage: {
      listenFilterMode(cb) {
        cb(initialMode);
        filterModeCb = cb;
        return () => undefined;
      },
      listenFilters(cb) {
        cb(initialFilters);
        filtersCb = cb;
        return () => undefined;
      },
      listenIncludeFocusedTabs(cb) {
        cb(initialIncludeFocusedTabs);
        includeFocusedTabsCb = cb;
        return () => undefined;
      },
    } as IFilterStorage,
  };
};
