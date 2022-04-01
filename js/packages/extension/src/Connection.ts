import { DEFAULT_CURRENT_PORT, DEFAULT_LEGACY_PORT, listenOption, Option } from './options';
import { formatLocalUrl } from '../../shared/url';
import { LegacyEventData, MessageCreator } from './types/message.types';
import { PlayInfo } from '../../shared/types';
import { IncomingMessages, OutgoingMessages, ReconnectingWebsocket } from '../../shared/reconnecting-websocket';

type ConnectionActiveMessage = LegacyEventData | PlayInfo;

export class Connection {
  private sock?: ReconnectingWebsocket<
    IncomingMessages,
    OutgoingMessages<{ Active: ConnectionActiveMessage; Inactive: undefined }>
  >;
  private isLegacy = false;
  private port = DEFAULT_CURRENT_PORT;

  private lastMessage: null | ConnectionActiveMessage = null;

  constructor() {
    // although this may not seem like it,
    // this will call the callback at the start.
    listenOption<boolean>(Option.UseLegacyApi, v => this.handleOptionChange(!!v, this.port));
    listenOption<string>(Option.ApiPort, v =>
      this.handleOptionChange(
        this.isLegacy,
        v ? Number(v) : this.isLegacy ? DEFAULT_LEGACY_PORT : DEFAULT_CURRENT_PORT,
      ),
    );
  }

  handleOptionChange(isLegacy: boolean, port: number) {
    const legacyChanged = this.isLegacy !== isLegacy;
    const portChanged = this.port !== port;
    if (!legacyChanged && !portChanged) return;

    if (this.sock) this.sock.close();
    if (legacyChanged) this.lastMessage = null;

    this.port = port;
    this.isLegacy = isLegacy;

    this.sock = new ReconnectingWebsocket(
      this.isLegacy ? `ws://127.0.0.1:${this.port}` : formatLocalUrl('/api/ws/extension', this.port, 'ws'),
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
