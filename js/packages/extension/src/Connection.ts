import { listenOption, Option } from './options';
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

  private lastMessage: null | ConnectionActiveMessage = null;

  constructor() {
    // although this may not seem like it,
    // this will call the callback at the start.
    listenOption<boolean | undefined>(Option.UseLegacyApi, v => this.handleOptionChange(!!v));
  }

  handleOptionChange(isLegacy: boolean) {
    if (this.sock) this.sock.close();
    if (this.isLegacy !== isLegacy) this.lastMessage = null;

    this.isLegacy = isLegacy;

    this.sock = new ReconnectingWebsocket(isLegacy ? 'ws://localhost:232' : formatLocalUrl('/api/ws/extension', 'ws'));
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
