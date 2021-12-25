import { ContextEventHandler } from 'beaverjs';
import { InternalMessageMap } from '../messages';
import { VideoPlayPosition } from '../types/video.types';
import { safeInject } from '../utils/safe-inject';

(async () => {
  if (!navigator.mediaSession) return;
  if (!safeInject('cso2:media-session-proxy')) return;

  const events = new ContextEventHandler<InternalMessageMap>();

  const proto: MediaSession = Object.getPrototypeOf(navigator.mediaSession);

  let positionState: Omit<VideoPlayPosition, 'mode'> | undefined,
    playbackState: MediaSessionPlaybackState | undefined,
    metadata: MediaMetadata | undefined | null;

  const onUpdate = () => {
    events.emitBackground('PlayMode', playbackState ?? 'none');
    if (metadata) {
      events.emitBackground('Metadata', {
        title: metadata.title,
        artist: metadata.artist,
        artwork: metadata.artwork?.[0]?.src, // TODO: get best image
      });
    }
    if (positionState) {
      events.emitBackground('PlayPosition', positionState);
    }
  };

  interceptFunction(proto, 'setPositionState', (state?: MediaPositionState) => {
    positionState = state ? toPlayPosition(state) : undefined;
    onUpdate();
  });

  interceptSet(proto, 'playbackState', (state: MediaSession['playbackState']) => {
    playbackState = state;
    onUpdate();
  });

  interceptSet(proto, 'metadata', (meta: MediaSession['metadata']) => {
    metadata = meta;
    onUpdate();
  });

  window.addEventListener('beforeunload', () => {
    events.emitBackground('PlayPosition', null);
    events.emitBackground('Metadata', null);
  });
})();

function toPlayPosition(pos: MediaPositionState): VideoPlayPosition {
  return {
    timestamp: Date.now(),
    rate: pos.playbackRate ?? 1,
    // both are given in seconds, we need milliseconds
    position: (pos.position ?? 0) * 1000,
    duration: (pos.duration ?? 0) * 1000,
  };
}

function interceptSet<K extends string, V>(target: { [x in K]: any }, key: K, fn: (value: V) => void) {
  const desc = Object.getOwnPropertyDescriptor(target, key);
  if (!desc) {
    console.error('Could not get property descriptor', target, key);
    return;
  }
  Object.defineProperty(target, key, {
    ...desc,
    set(value: V) {
      fn(value);
      desc.set?.call(this, value);
    },
  });
}

function interceptFunction<K extends string, V extends (...args: any[]) => any>(
  target: { [x in K]?: V },
  key: K,
  fn: (...args: Parameters<V>) => void,
) {
  const base = target[key];
  if (!base) {
    return;
  }
  target[key] = function (...args: Parameters<V>): ReturnType<V> {
    // Here a `this` is used.
    // Since typescript doesn't know the `this` context in the anonymous function,
    // we're aliasing `this` once, so we only need 2 comments.

    // @ts-ignore -- this is any, that's fine
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const that = this;
    fn.apply(that, args);
    return base.apply(that, args);
  } as V;
}
