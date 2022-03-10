import '../chrome-fix';
import './upgrade';
import { BackgroundEventHandler } from 'beaverjs';
import { InternalMessageMap } from '../messages';
import { TabManager } from './TabManager';
import { Connection } from '../Connection';
import { FilterManager } from '../filters/FilterManager';
import { LocalFilterStorage } from '../filters/FilterStorage';
import { DefaultBrowserInterface } from './BrowserInterface';

(async () => {
  const connection = new Connection();

  const manager = await createManager(connection);

  const events = new BackgroundEventHandler<InternalMessageMap>();
  events.on('PlayPosition', (data, sender) => {
    if (sender.tab) manager.setPlayPosition(sender.tab, data || undefined);
  });
  events.on('Metadata', (data, sender) => {
    if (sender.tab) manager.setMetadata(sender.tab, data || undefined);
  });
})();

async function createManager(connection: Connection): Promise<TabManager> {
  const initialWindows = await browser.windows.getAll({ populate: true });

  return new TabManager({
    initialTabs: initialWindows.map(w => w.tabs || []).flat(),
    updateCallback: message => {
      console.trace('%cSend %o', 'color: red; font-weight: bold; font-size: 2em;', message);
      connection.send(message ?? undefined);
    },
    initialWindows,
    filterManager: new FilterManager(LocalFilterStorage),
    browser: DefaultBrowserInterface,
  });
}
