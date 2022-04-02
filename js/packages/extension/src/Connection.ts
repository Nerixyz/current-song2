import { DEFAULT_CURRENT_PORT, DEFAULT_LEGACY_PORT, listenOption, Option } from './options';
import { formatLocalUrl } from '../../shared/url';
import { LegacyEventData, MessageCreator } from './types/message.types';
import { PlayInfo } from '../../shared/types';
import { IncomingMessages, OutgoingMessages, ReconnectingWebsocket } from '../../shared/reconnecting-websocket';

type ConnectionActiveMessage = LegacyEventData | PlayInfo;
type Port = number | undefined | null;

export class Connection {
  private sock?: ReconnectingWebsocket<
    IncomingMessages,
    OutgoingMessages<{ Active: ConnectionActiveMessage; Inactive: undefined }>
  >;
  private isLegacy = false;
  private port: Port = null;

  private lastMessage: null | ConnectionActiveMessage = null;

  constructor() {
    // although this may not seem like it,
    // this will call the callback at the start.
    listenOption<boolean>(Option.UseLegacyApi, v => this.handleOptionChange(!!v, this.port));
    listenOption<string>(Option.ApiPort, v => this.handleOptionChange(this.isLegacy, v ? Number(v) : null));
  }

  handleOptionChange(isLegacy: boolean, port: Port) {
    const legacyChanged = this.isLegacy !== isLegacy;
    const portChanged = this.port !== port;
    if (!legacyChanged && !portChanged && this.sock) return;

    if (this.sock) this.sock.close();
    if (legacyChanged) this.lastMessage = null;

    this.port = port;
    this.isLegacy = isLegacy;

    const actualPort = this.port ?? (this.isLegacy ? DEFAULT_LEGACY_PORT : DEFAULT_CURRENT_PORT);
    this.sock = new ReconnectingWebsocket(
      this.isLegacy ? `ws://127.0.0.1:${actualPort}` : formatLocalUrl('/api/ws/extension', actualPort, 'ws'),
    );
    this.sock.connect().then(() => {
      if (this.lastMessage) {
        this.sock?.trySend('Active', this.lastMessage);
      }
    });
  }

  /**
   * @param {MessageCreator | undefined} creator `undefined` implies 'Inactive'
   */
  send(creator?: MessageCreator) {
    if (this.sock) {
      if (creator) {
        const message = this.isLegacy ? creator.createLegacyEvent() : creator.createPlayInfo();
        this.lastMessage = message;

        this.sock.trySend('Active', message);
      } else {
        this.lastMessage = null;
        this.sock.trySend('Inactive', undefined);
      }
    }
  }
}
