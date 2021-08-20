/**
 * Gets all elements by their id. These elements _must_ exist.
 * @param {...string} ids
 * @returns {A}
 */
export function getElements<A extends Array<HTMLElement>>(...ids: string[]): A {
  // This is unsafe but we know these elements exist when calling the function
  return ids.map(id => {
    const el = document.getElementById(id);
    if (el === null) throw new Error(`${id} does not exist`);

    return el;
  }) as A;
}
