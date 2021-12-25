browser.runtime.onInstalled.addListener(({ reason, previousVersion }) => {
  if (reason === 'update') {
    const isOldVersion = /0\.[0-5]\.\d+/.test(previousVersion ?? '0.6.0');
    if (isOldVersion) {
      browser.storage.local.set({ 'use-legacy-api': true }).catch(console.error);
    }
  }
});
