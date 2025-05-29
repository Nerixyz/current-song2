import { getElements } from './dom/utils';
import { createProgress } from './progress';
import { smolTree } from './dom/smol-tree';
import {
  hasAlbum,
  hasImage,
  hasSubtitle,
  hasTimeline,
  hasTrack,
  hasValidAlbumTracks,
  isSpotify,
  makeState,
  not,
  State,
} from './state';
import { animateOnChange, TextChangeAnimation } from './dom/animation';
import { EventMap } from './types';
import { formatLocalUrl } from '../../shared/url';
import {
  IncomingMessages,
  OutgoingMessages,
  ReconnectingWebsocket,
} from '../../shared/reconnecting-websocket';
import { makeCso2Env, startUserScript } from './user-scripts';
import { MarqueeEl, MarqueeOptions, wrapMarquee } from './text/marquee';
import { setupOptions } from './options';

// setup environment for plugins
(globalThis as any).cso2 = makeCso2Env();

function wrapMarqueeElements(
  root: HTMLElement,
  titleEl: HTMLElement,
  subtitleEl: HTMLElement,
): MarqueeEl {
  const style = getComputedStyle(root);
  const useIt = style.getPropertyValue('--use-marquee').trim() === 'true';
  if (!useIt) {
    return { pause() {}, reset() {}, start() {} };
  }
  const opt = (name: string, defaultValue: number) => {
    const parsed = parseFloat(style.getPropertyValue(name).trim());
    return Number.isNaN(parsed) ? defaultValue : parsed;
  };
  const opts: MarqueeOptions = {
    speed: opt('--marquee-speed', 0.2),
    pauseDuration: opt('--marquee-pause-duration', 1200),
    repeatPauseDuration: opt('--marquee-repeat-pause-duration', 2000),
  };
  const title = wrapMarquee(titleEl, opts);
  const subtitle = wrapMarquee(subtitleEl, opts);

  return {
    pause: () => {
      title.pause();
      subtitle.pause();
    },
    start: () => {
      title.start();
      subtitle.start();
    },
    reset: () => {
      title.reset();
      subtitle.reset();
    },
  };
}

(async function main() {
  const [container, imageContainer, imageEl, titleEl, subtitleEl, progressEl] = getElements<
    [
      HTMLDivElement,
      HTMLDivElement,
      HTMLImageElement,
      HTMLHeadingElement,
      HTMLHeadingElement,
      HTMLDivElement,
    ]
  >('song-container', 'image-container', 'image', 'title', 'subtitle', 'progress');
  const resetMarquee = wrapMarqueeElements(container, titleEl, subtitleEl);

  const progressManager = createProgress(progressEl);

  const tree = smolTree<State>(
    [imageEl, { spotify: isSpotify }],
    [imageContainer, { hidden: not(hasImage) }],
    [
      container,
      {
        'with-image': hasImage,
        'is-spotify': isSpotify,
        'with-progress': hasTimeline,
        'has-album-tracks': hasValidAlbumTracks,
        'has-track': hasTrack,
        'with-album': hasAlbum,
      },
    ],
    [subtitleEl, { hidden: not(hasSubtitle) }],
  );
  const scriptOptions = setupOptions();

  const userScript = startUserScript();

  const ws = new ReconnectingWebsocket<IncomingMessages<EventMap>, OutgoingMessages>(
    formatLocalUrl({
      path: '/api/ws/client',
      port: Number(location.port) || 48457,
      protocol: 'ws',
      host: location.hostname,
    }),
  );
  ws.addEventListener('Playing', ({ data }) => {
    container.classList.remove('vanish');
    const state = makeState(data, scriptOptions);
    tree.update(state);
    resetMarquee.start();

    animateOnChange(titleEl, state.title, resetMarquee.reset, ...TextChangeAnimation);
    if (state.subtitle) {
      animateOnChange(subtitleEl, state.subtitle, resetMarquee.reset, ...TextChangeAnimation);
    }

    if (state.imageUrl) {
      imageEl.src = state.imageUrl;
      container.style.setProperty('--image-url', `url("${encodeURI(state.imageUrl)}")`);
    }
    container.style.setProperty('--title', JSON.stringify(data.title));
    container.style.setProperty('--artist', JSON.stringify(data.artist));
    if (data.album) {
      container.style.setProperty('--album', JSON.stringify(data.album.title));
      container.style.setProperty(
        '--album-tracks',
        JSON.stringify(data.album.trackCount.toString()),
      );
    }
    if (data.trackNumber) {
      container.style.setProperty('--track-number', JSON.stringify(data.trackNumber.toString()));
    }

    progressManager.run(data.timeline);

    userScript.onPlay(state);
  });
  ws.addEventListener('Paused', () => {
    container.classList.add('vanish');
    progressManager.pause();

    userScript.onPause();
    resetMarquee.pause();
  });
  await ws.connect();
})();
