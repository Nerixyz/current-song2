# Display API

The API used by the overlay to display the current song is open for developers to use.
Note that only the display API is considered open.

To receive updates, connect using a WebSocket to `ws://localhost:48457/api/ws/client` (when using a [custom port](Configuration.md#port), use that one instead).

Here, all types are given in Typescript syntax.

## Messages

All messages are JSON encoded and follow the structure `{ type, data }` where `type` dictates the type of `data`. When connecting to the server, you'll receive a fresh state.

```ts
type Message =
    | /* (1)! */ {
          type: 'Playing';
          data: PlayInfo; // (2)!
      }
    | {
          type: 'Paused';
      }
    | {
          type: 'Ping';
      }
    | {
          type: 'Pong'; // (3)!
      };
```

1. The pipe (`|`) means _OR_ - i.e. a message will be exactly one of the objects.
2. Described in the [types](#types) section.
3. You will never receive this - you'll send this message.

When receiving a `Ping` message, you must immediately respond with a `Pong` message.

## Types

### `PlayInfo`

`PlayInfo` is a core type encapsulating the entire state:

```ts
interface PlayInfo {
    title: string;
    artist: string; // (1)!
    trackNumber: null | number;

    image: null | ImageInfo;
    timeline: null | TimelineInfo;
    album: null | AlbumInfo;

    source: string; // (2)!
}
```

1. The artist might be an empty string.
2. The source is a hint on where Current Song 2 got the information from. For GSMTC, it will be formatted like `gsmtc::<executable>`. This might be useful to detect some applications like Spotify.

### `ImageInfo`

An image can either be a URL (`string`) or an image hosted on the local server (`InternalImage`). To get the URL for an internal image, use `http://localhost:48457/api/img/{id}/{epochId}` (when using a [custom port](Configuration.md#port), use that one instead).

```ts
type ImageInfo = string | InternalImage;

interface InternalImage {
    id: number;
    epochId: number;
}
```

### `TimelineInfo`

```ts
interface TimelineInfo {
    ts: number; // (1)!
    durationMs: number; // (2)!
    progressMs: number; // (3)!
    rate: number; // (4)!
}
```

1. The UTC timestamp in milliseconds when the timeline info was captured.
2. The duration of the current song in milliseconds.
3. The playback position (at the timestamp) in milliseconds (from the start - 0ms).
4. The playback rate of the song.

### `AlbumInfo`

```ts
interface AlbumInfo {
    title: string;
    trackCount: number; // (1)!
}
```

1. The track count might not always be available in which case it will be `0`.

## Usability Considerations

To deliver a good user experience, consider the following.

### Reconnecting

Ensure that your integration can handle potential restarts of the server by reconnecting (with a backoff) when the connection drops.
