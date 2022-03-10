/**
 * runs multiple functions with queueMicrotask after each other
 * @param {() => void} fns
 */
export function queueMicrotaskChain(...fns: Array<() => void>) {
  let idx = 0;
  const next = () => {
    if (idx < fns.length) {
      queueMicrotask(() => {
        fns[idx]();
        idx++;
        next();
      });
    }
  };
  next();
}

export function waitMicrotask() {
  return new Promise<void>(res => queueMicrotask(res));
}

export function waitNMicrotasks(n: number) {
  return new Promise<void>(res => {
    const fns: Array<() => void> = [];
    for (let i = 0; i < n - 1; i++) {
      fns.push(() => undefined);
    }
    fns.push(res);
    queueMicrotaskChain(...fns);
  });
}
