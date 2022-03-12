import { FilterManager, matchModel, tryParseUrl } from '../../src/filters/FilterManager';
import { IFilterStorage, RevokeListener } from '../../src/filters/FilterStorage';
import { DEFAULT_FILTER_MODE, FilterMode } from '../../src/options';
import { FilterModel } from '../../src/filters/types';
import { staticStorage } from '../mock/filter-storage';

describe('FilterManager', function () {
  it('should support blocking filtering', function () {
    const Storage = staticStorage(
      [
        { value: 'github.com', isRegex: false },
        { value: '^http://', isRegex: true },
      ],
      FilterMode.Block,
    );
    const filter = new FilterManager(Storage);
    expect(filter.checkUrl('https://github.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('http://github.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('http://gitlab.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('https://github.com')).toBe(false);
    expect(filter.checkUrl('http://github.com')).toBe(false);

    expect(filter.checkUrl('https://gitlab.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('ftp://gitlab.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('https://gitlab.com')).toBe(true);
    expect(filter.checkUrl('https://forsen.tv')).toBe(true);
    expect(filter.checkUrl('http//://invalid')).toBe(true);
  });

  it('should support allow filtering', function () {
    const Storage = staticStorage(
      [
        { value: 'github.com', isRegex: false },
        { value: '^http://', isRegex: true },
      ],
      FilterMode.Allow,
    );
    const filter = new FilterManager(Storage);
    expect(filter.checkUrl('https://github.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('http://github.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('http://gitlab.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('https://github.com')).toBe(true);
    expect(filter.checkUrl('http://github.com')).toBe(true);

    expect(filter.checkUrl('https://gitlab.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('ftp://gitlab.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('https://gitlab.com')).toBe(false);
    expect(filter.checkUrl('https://forsen.tv')).toBe(false);
    expect(filter.checkUrl('http//://invalid')).toBe(false);
  });

  it('should handle a full lifecycle', function () {
    const revokeFilters = jest.fn();
    const revokeFilterMode = jest.fn();

    let setFilters = (_filters?: FilterModel[]): void => void 0;
    let setFilterMode = (_mode?: FilterMode): void => void 0;
    const listenFilters = jest.fn((cb: (filters?: FilterModel[]) => void): RevokeListener => {
      setFilters = cb;
      return revokeFilters;
    });
    const listenFilterMode = jest.fn((cb: (mode?: FilterMode) => void): RevokeListener => {
      setFilterMode = cb;
      return revokeFilterMode;
    });
    const onUpdate = jest.fn();
    const Storage: IFilterStorage = { listenFilters, listenFilterMode };
    const filter = new FilterManager(Storage, onUpdate);

    // Part 1: check initial state
    expect(listenFilters).toBeCalledTimes(1);
    expect(listenFilterMode).toBeCalledTimes(1);
    expect(onUpdate).toBeCalledTimes(0);
    expect(revokeFilters).toBeCalledTimes(0);
    expect(revokeFilterMode).toBeCalledTimes(0);

    // Part 2.0: check onUpdate with setFilterMode
    setFilterMode();
    expect(onUpdate).toBeCalledTimes(0);
    setFilterMode(DEFAULT_FILTER_MODE);
    expect(onUpdate).toBeCalledTimes(0);
    setFilterMode(DEFAULT_FILTER_MODE === FilterMode.Block ? FilterMode.Allow : FilterMode.Block);
    expect(onUpdate).toBeCalledTimes(1);

    // Part 2.1: check onUpdate with setFilters
    //           since arrays are not checked for deep equality, we skip one test
    setFilters();
    expect(onUpdate).toBeCalledTimes(1);
    setFilters([{ value: 'github.com', isRegex: false }]);
    expect(onUpdate).toBeCalledTimes(2);

    // Part 3: check updates to filterMode and filters
    setFilterMode(FilterMode.Allow);
    expect(filter.checkUrl('https://github.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('https://gitlab.com/notifications?query=#forsen')).toBe(false);
    setFilterMode(FilterMode.Block);
    expect(filter.checkUrl('https://github.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('https://gitlab.com/notifications?query=#forsen')).toBe(true);
    setFilters([]);
    expect(filter.checkUrl('https://github.com/notifications?query=#forsen')).toBe(true);
    expect(filter.checkUrl('https://gitlab.com/notifications?query=#forsen')).toBe(true);
    setFilterMode(FilterMode.Allow);
    expect(filter.checkUrl('https://github.com/notifications?query=#forsen')).toBe(false);
    expect(filter.checkUrl('https://gitlab.com/notifications?query=#forsen')).toBe(false);

    // Part 4.0: make sure nothing else was called
    expect(listenFilters).toBeCalledTimes(1);
    expect(listenFilterMode).toBeCalledTimes(1);
    expect(revokeFilters).toBeCalledTimes(0);
    expect(revokeFilterMode).toBeCalledTimes(0);

    // Part 4.1: check closing
    filter.close();
    expect(listenFilters).toBeCalledTimes(1);
    expect(listenFilterMode).toBeCalledTimes(1);
    expect(revokeFilters).toBeCalledTimes(1);
    expect(revokeFilterMode).toBeCalledTimes(1);
  });
});

describe('tryParseUrl', function () {
  it('should parse urls correctly', function () {
    expect(tryParseUrl('http://localhost:8080')).toStrictEqual(new URL('http://localhost:8080'));
    expect(tryParseUrl('localhost:8080')).toStrictEqual(new URL('localhost:8080'));
    expect(tryParseUrl('https://github.com/notifications?query=#forsen')).toStrictEqual(
      new URL('https://github.com/notifications?query=#forsen'),
    );
    expect(tryParseUrl('github.com/notifications?query=#forsen')).toStrictEqual(
      new URL('https://github.com/notifications?query=#forsen'),
    );
    expect(tryParseUrl('http//://invalid')).toStrictEqual(null);
  });
});

describe('matchModel', function () {
  const regularModel = (url: string, value: string) => matchModel(new URL(url), { value, isRegex: false });
  const regexModel = (url: string, value: string) => matchModel(new URL(url), { value, isRegex: true });

  it('should match regular filters', function () {
    expect(regularModel('https://github.com/notifications?query=#forsen', 'github.com')).toBe(true);
    expect(regularModel('https://github.com/notifications?query=#forsen', 'git')).toBe(true);
    expect(regularModel('https://github.com/notifications?query=#forsen', 'hub.com')).toBe(true);
    expect(regularModel('https://github.com/notifications?query=#forsen', '.com')).toBe(true);
    expect(regularModel('https://github.com/notifications?query=#forsen', 'com')).toBe(true);

    expect(regularModel('https://github.com/notifications?query=#forsen', 'hub')).toBe(false);
    expect(regularModel('https://github.com/notifications?query=#forsen', 'https://github.com')).toBe(false);
    expect(regularModel('https://github.com/notifications?query=#forsen', 'forsen')).toBe(false);
    expect(regularModel('https://github.com/notifications?query=#forsen', 'https')).toBe(false);
  });

  it('should match regex filters', function () {
    // the tests from above should also be covered
    expect(regexModel('https://github.com/notifications?query=#forsen', 'github.com')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'git')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'hub.com')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', '.com')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'com')).toBe(true);

    // in contrast to the regular model, here the whole url is matched
    expect(regexModel('https://github.com/notifications?query=#forsen', 'hub')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'https://github.com')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'forsen')).toBe(true);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'https')).toBe(true);

    // TODO: figure out good tests
    expect(regexModel('https://github.com/notifications?query=#forsen', '^ftp://')).toBe(false);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'gitlab\\.com')).toBe(false);
    expect(regexModel('https://github.com/notifications?query=#forsen', '^forsen')).toBe(false);
    expect(regexModel('https://github.com/notifications?query=#forsen', 'https$')).toBe(false);
  });
});
