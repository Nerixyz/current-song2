# User Scripts

In addition to custom themes, you can customize the overlay using JavaScript through a `user.js` file (or a different filename specified in [`server.custom_script_path`](../Configuration.md#custom_script_path)).

The script is loaded at the start.
In your script, you can expose `onPlay(state)` and `onPause()` through exports which will get called at the appropriate event.

`state` has the following type:

```ts
interface State {
    info: PlayInfo;
    title: string;
    subtitle: string | undefined;
    imageUrl: string | undefined;
}

interface PlayInfo {
    title: string;
    artist: string;
    trackNumber: null | number;

    image: null | ImageInfo;
    timeline: null | TimelineInfo;
    album: null | AlbumInfo;

    source: string;
}

type ImageInfo = string | InternalImage;

interface InternalImage {
    id: number;
    epochId: number;
}

interface TimelineInfo {
    ts: number;
    durationMs: number;
    progressMs: number;
    rate: number;
}
```

Additionally, user scripts have access to the `cso2` global which contains helper functions. Currently, there's only a function to wrap elements in a marquee container.

```ts
declare var cso2: CSO2;
interface CSO2 {
    marquee: MarqueeLib;
}

interface MarqueeOptions {
    speed: number;
    pauseDuration: number;
    repeatPauseDuration: number;
}

interface MarqueeEl {
    pause: () => void;
    start: () => void;
    reset: () => void;
}

interface MarqueeLib {
    wrap: (el: HTMLElement, opts?: MarqueeOptions) => MarqueeEl;
}
```

Example:

```javascript title="user.js"
console.log('Hello, World!');

export function onPlay(state) {
    console.log('Hello, State!', state);
}

export function onPause() {
    console.log('Hello, Pause!');
}
```

See [Examples](Theming/Examples.md) for a more involved example of adding an artist line.
