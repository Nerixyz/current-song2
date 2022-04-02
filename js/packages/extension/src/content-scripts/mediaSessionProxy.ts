import '../chrome-fix';
import { ContentEventHandler } from 'beaverjs';

if (document.documentElement instanceof HTMLElement) {
  try {
    // act as a proxy
    // we only want to pass events to the background
    new ContentEventHandler(['context']);
    const el = document.createElement('script');
    el.src = browser.runtime.getURL('content-scripts/mediaSessionProxy.inject.js');
    el.onload = () => el.remove();
    document.documentElement.prepend(el);
  } catch (e) {
    console.error(e);
  }
} else {
  console.debug("%c[CSO2] %cDocument isn't an HTML Document, skipping injection.", 'color: red', 'color: yellow');
}
