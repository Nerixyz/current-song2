export function* filter<T>(iter: Iterable<T>, fn: (value: T) => boolean) {
  for (const item of iter) {
    if (fn(item)) yield item;
  }
}

export function first<T>(iter: Iterable<T>): T | null {
  const { value, done } = iter[Symbol.iterator]().next();
  return done ? null : value;
}
