if (document.documentElement instanceof HTMLElement) {
  const el = document.createElement('script');
  el.src = browser.runtime.getURL('content-scripts/mediaSessionProxy.inject.js');
  el.onload = () => el.remove();
  document.body.prepend(el);
} else {
  console.debug("%c[CSO2] %cDocument isn't an HTML Document, skipping injection.", 'color: red', 'color: yellow');
}
