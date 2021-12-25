import { ContentEventHandler } from 'beaverjs';
import { InternalMessageMap } from '../messages';
import { VideoPlayPosition } from '../types/video.types';

function changeHandler({ target }: { target: unknown }) {
  if (!(target instanceof HTMLVideoElement)) return;
  sendCurrent(target).catch(console.error);
}

const events = new ContentEventHandler<InternalMessageMap>();

document.addEventListener('playing', changeHandler, true);
document.addEventListener('pause', changeHandler, true);
document.addEventListener('ratechange', changeHandler, true);
document.addEventListener('seeked', changeHandler, true);

function createPosition(target: HTMLVideoElement): VideoPlayPosition {
  return {
    rate: target.playbackRate,
    timestamp: Date.now(),
    // both are in seconds, we need milliseconds.
    duration: target.duration * 1000,
    position: target.currentTime * 1000,
  };
}

async function sendCurrent(target: HTMLVideoElement) {
  const duration = await waitUntilNotNaN(() => target.duration);

  if (duration === null) {
    console.error('Even after waiting for video, "duration" is still NaN', target);
    return;
  }

  events.emitBackground('PlayPosition', createPosition(target));
  events.emitBackground('PlayMode', target.paused ? 'paused' : 'playing');
}

async function waitUntilNotNaN(fn: () => number, tries = 20, delay = 100): Promise<number | null> {
  let counter = 0;
  let n = fn();
  while (Number.isNaN(n) && counter < tries) {
    await new Promise(resolve => setTimeout(resolve, delay));

    counter++;
    n = fn();
  }
  return Number.isNaN(n) ? null : n;
}
