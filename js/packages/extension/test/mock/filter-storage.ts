import { FilterModel } from '../../src/filters/types';
import { FilterMode } from '../../src/options';
import { IFilterStorage } from '../../src/filters/FilterStorage';

export const staticStorage = (filters: FilterModel[], mode: FilterMode): IFilterStorage => ({
  listenFilterMode(cb) {
    cb(mode);
    return () => undefined;
  },
  listenFilters(cb) {
    cb(filters);
    return () => undefined;
  },
});

export const dynamicStorage = (
  initialFilters: FilterModel[],
  initialMode: FilterMode,
): { setFilters(filters: FilterModel[]): void; setMode(mode: FilterMode): void; filterStorage: IFilterStorage } => {
  let filterModeCb: null | ((mode: FilterMode) => void) = null;
  let filtersCb: null | ((filters: FilterModel[]) => void) = null;
  return {
    setFilters: (filters: FilterModel[]) => queueMicrotask(() => filtersCb?.(filters)),
    setMode: (mode: FilterMode) => queueMicrotask(() => filterModeCb?.(mode)),
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
    },
  };
};
