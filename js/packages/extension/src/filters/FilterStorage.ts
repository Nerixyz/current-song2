import { FilterMode, listenJsonOption, listenOption, Option } from '../options';
import { FilterModel } from './types';

export type RevokeListener = () => void;
export interface IFilterStorage {
  listenFilterMode(cb: (mode?: FilterMode) => void): RevokeListener;
  listenFilters(cb: (filters?: FilterModel[]) => void): RevokeListener;
  listenIncludeFocusedTabs(cb: (includeFocusedTabs?: boolean) => void): RevokeListener;
}

export const LocalFilterStorage: IFilterStorage = {
  listenFilterMode(cb: (mode?: FilterMode) => void): RevokeListener {
    return listenOption(Option.FilterMode, cb);
  },
  listenFilters(cb: (filters?: FilterModel[]) => void): RevokeListener {
    return listenJsonOption(Option.Filters, cb);
  },
  listenIncludeFocusedTabs(cb: (includeFocusedTabs?: boolean) => void): RevokeListener {
    return listenOption(Option.IncludeFocusedTabs, cb);
  },
};
