/**
 * @returns {boolean} true if the host doesn't have promise-like apis
 */
export function isChromeLike() {
  // returns 1 on Firefox and 0 on Chrome
  // => false on Firefox; true on Chrome
  return !browser.tabs.detectLanguage.length;
}
