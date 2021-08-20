import { EventMap } from './types';
import { formatLocalUrl } from './url';

export class WsMessageEvent<K extends keyof EventMap> extends Event {
  constructor(public messageType: K, public data: EventMap[K]) {
    super(messageType);
  }
}

export interface ReconnectingWebsocket extends EventTarget {
  addEventListener<K extends keyof EventMap>(
    type: K,
    listener: (this: WebSocket, ev: WsMessageEvent<K>) => any,
    options?: boolean | AddEventListenerOptions,
  ): void;
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener<K extends keyof EventMap>(
    type: K,
    listener: (this: WebSocket, ev: WsMessageEvent<K>) => any,
    options?: boolean | EventListenerOptions,
  ): void;
  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | EventListenerOptions,
  ): void;
}

export class ReconnectingWebsocket extends EventTarget {
  private ws?: WebSocket;
  private nextDelay = 1;

  public connect(): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const ws = new WebSocket(createWebsocketUrl());
      this.ws = ws;

      // functions to make the first connection a promise
      const clearListeners = () => {
        ws.removeEventListener('open', openListener);
        ws.removeEventListener('error', errorListener);
        clearTimeout(timeoutId);
      };
      const openListener = () => {
        resolve();
        clearListeners();
      };
      const errorListener = (e: Event) => {
        reject(e);
        clearListeners();
      };

      // the actual handlers
      ws.addEventListener('open', openListener);
      ws.addEventListener('error', errorListener);

      const closeListener = () => {
        ws.removeEventListener('message', messageListener);
        this.reconnectLater();
      };
      const messageListener = (e: MessageEvent) => {
        if (typeof e.data === 'string') {
          try {
            const json = JSON.parse(e.data);
            if (typeof json === 'object' && json !== null && typeof json.type === 'string') {
              this.handleMessage(json.type, json.data);
            } else {
              console.warn('invalid messgae', json);
            }
          } catch (e) {
            console.warn('invalid json', e);
          }
        }
      };
      ws.addEventListener('close', closeListener, { once: true });
      ws.addEventListener('message', messageListener);

      const timeoutId = setTimeout(() => {
        ws.close();
        reject('Connection Timed Out');
      }, 2000);
    });
  }

  private reconnectLater() {
    console.info(`Reconnecting in ${this.nextDelay}s`);
    setTimeout(() => {
      this.nextDelay = Math.min(this.nextDelay * 2 + 5, 120);
      this.connect().then(() => (this.nextDelay = 1));
    }, this.nextDelay * 1000);
  }

  private handleMessage<K extends keyof EventMap>(type: K, content: EventMap[K]) {
    this.dispatchEvent(new WsMessageEvent(type, content));

    if (type === 'Ping') {
      this.ws?.send(JSON.stringify({ type: 'Pong' }));
    }
  }
}

function createWebsocketUrl() {
  return formatLocalUrl('/api/ws/client', 'ws');
}
