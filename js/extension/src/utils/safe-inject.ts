/**
 * Makes sure there's at most one script active
 * @param {string} key
 * @returns {boolean} true if this is the first time the script is injected.
 */
export function safeInject(key: string) {
  const idKey = Symbol.for(key);

  if ((globalThis as any)[idKey]) {
    console.debug('%c[CSO2] %cSkipping injection since the script is already injected.', 'color: red', 'color: yellow');
    return false;
  }

  Object.defineProperty(globalThis, idKey, {
    configurable: false,
    enumerable: false,
    writable: false,
    value: true,
  });

  return true;
}
