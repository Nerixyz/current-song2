import { getElements } from './dom/utils';
import { ReconnectingWebsocket } from './reconnecting-websocket';
import { createProgress } from './progress';
import { smolTree } from './dom/smol-tree';
import { hasImage, hasSubtitle, hasTimeline, isSpotify, makeState, not, State } from './state';
import { animateOnChange, TextChangeAnimation } from './dom/animation';

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

  const progressManager = createProgress(progressEl);

  const tree = smolTree<State>(
    [imageEl, { spotify: isSpotify }],
    [imageContainer, { hidden: not(hasImage) }],
    [container, { 'with-image': hasImage, 'with-progress': hasTimeline }],
    [subtitleEl, { hidden: not(hasSubtitle) }],
  );

  const ws = new ReconnectingWebsocket();
  ws.addEventListener('Playing', ({ data }) => {
    container.classList.remove('vanish');
    const state = makeState(data);
    tree.update(state);

    animateOnChange(titleEl, state.title, ...TextChangeAnimation);
    if (state.subtitle) animateOnChange(subtitleEl, state.subtitle, ...TextChangeAnimation);

    if (state.imageUrl) imageEl.src = state.imageUrl;

    progressManager.run(data.timeline);
  });
  ws.addEventListener('Paused', () => {
    container.classList.add('vanish');
  });
  await ws.connect();
})();
