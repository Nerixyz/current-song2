export function find<T>(iter: Iterable<T>, fn: (item: T) => boolean) {
  for (const item of iter) {
    if (fn(item)) return item;
  }
  return null;
}
