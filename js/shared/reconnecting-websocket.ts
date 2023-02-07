export type MinEventMap = { [x: string]: unknown } & { Ping: undefined };
export type MinSendMap = { [x: string]: unknown } & { Pong: undefined };

export type IncomingMessages<T = Record<string, never>> = T & { Ping: undefined };
export type OutgoingMessages<T = Record<string, never>> = T & { Pong: undefined };

export class WsMessageEvent<M extends { [x: string]: unknown }, K extends keyof M> extends Event {
  public messageType: K;

  constructor(messageType: K, public data: M[K]) {
    super(messageType as any);
    this.messageType = messageType;
  }
}

// Provide types for the events.
export interface ReconnectingWebsocket<EventMap extends MinEventMap, SendMap extends MinSendMap> extends EventTarget {
  addEventListener<K extends keyof EventMap>(
    type: K,
    listener: (this: WebSocket, ev: WsMessageEvent<EventMap, K>) => any,
    options?: boolean | AddEventListenerOptions,
  ): void;

  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | AddEventListenerOptions,
  ): void;

  removeEventListener<K extends keyof EventMap>(
    type: K,
    listener: (this: WebSocket, ev: WsMessageEvent<EventMap, K>) => any,
    options?: boolean | EventListenerOptions,
  ): void;

  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | EventListenerOptions,
  ): void;
}

/**
 * A managed WebSocket which automatically reconnects.
 *
 * @example
 * ```ts
 * const sock = new ReconnectingWebsocket<IncomingMessages<{NewMessage: string}>, OutgoingMessages<{SendMessage: string}>>('ws://localhost:8080');
 *
 * sock.addEventListener('NewMessage', console.log);
 * await sock.connect();
 * sock.trySend('SendMessage', 'Connected');
 * ```
 */
export class ReconnectingWebsocket<EventMap extends MinEventMap, SendMap extends MinSendMap> extends EventTarget {
  private ws?: WebSocket;
  private nextDelay = 1;
  private shouldClose = false;

  constructor(private url: string | URL) {
    super();
  }

  public connect(): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const ws = new WebSocket(this.url);
      this.ws = ws;
      this.shouldClose = false;
      console.info(`Connecting to ${this.ws.url}`);

      // functions to make the first connection a promise
      const clearListeners = () => {
        ws.removeEventListener('open', openListener);
        ws.removeEventListener('error', errorListener);
        clearTimeout(timeoutId);
      };
      const openListener = () => {
        resolve();
        console.info(`Connected to ${this.ws?.url ?? '<?>'}`);
        clearListeners();
      };
      const errorListener = (e: Event | ErrorEvent) => {
        reject(
          (e as ErrorEvent).message
            ? (e as ErrorEvent).message
            : `Connection closed readyState: ${(e.target as WebSocket).readyState}`,
        );
        clearListeners();
      };
      const closeListener = () => {
        ws.removeEventListener('message', messageListener);
        if (!this.shouldClose) this.reconnectLater();
      };
      const messageListener = (e: MessageEvent) => {
        if (typeof e.data !== 'string') return;

        try {
          const json = JSON.parse(e.data);
          if (typeof json === 'object' && json !== null && typeof json.type === 'string') {
            this.handleMessage(json.type, json.data);
          } else {
            console.warn('invalid message', json);
          }
        } catch (e) {
          console.warn('invalid json', e);
        }
      };

      // the actual handlers
      ws.addEventListener('open', openListener);
      ws.addEventListener('error', errorListener);
      ws.addEventListener('close', closeListener, { once: true });
      ws.addEventListener('message', messageListener);

      const timeoutId = setTimeout(() => {
        ws.close();
        reject('Connection Timed Out');
      }, 2000);
    });
  }

  public trySend<K extends keyof SendMap>(type: K, data: SendMap[K]) {
    try {
      this.ws?.send(JSON.stringify({ type, data }));
    } catch (e) {
      console.warn('Error sending websocket message:', e);
    }
  }

  public close() {
    this.shouldClose = true;
    this.ws?.close();
    this.ws = undefined;
  }

  private reconnectLater() {
    console.info(`Reconnecting in ${this.nextDelay}s`);
    setTimeout(() => {
      this.nextDelay = Math.min(this.nextDelay * 2 + 3, 120);

      if (!this.shouldClose) this.connect().then(() => (this.nextDelay = 1));
    }, this.nextDelay * 1000);
  }

  private handleMessage<K extends keyof EventMap>(type: K, content: EventMap[K]) {
    this.dispatchEvent(new WsMessageEvent(type, content));

    if (type === 'Ping') {
      this.trySend('Pong', undefined);
    }
  }
}
