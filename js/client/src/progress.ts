import { TimelineInfo } from './types';

export function createProgress(element: HTMLDivElement): {
  run: (timeline: TimelineInfo | null) => void;
  pause: () => void;
} {
  let running = false;
  let currentTl: TimelineInfo | null = null;
  let progress = -1;

  const runTick = () => {
    if (!currentTl) {
      element.classList.add('hidden');
      running = false;
      return;
    }
    if (!running) {
      return;
    }

    const newProgress = progressNow(currentTl);
    if (progress !== newProgress) {
      progress = newProgress;
      document.body.style.setProperty('--progress', progress.toString());
    }
    requestAnimationFrame(runTick);
  };

  return {
    run: timeline => {
      currentTl = timeline;
      element.classList.remove('hidden');
      element.classList.remove('paused');

      if (!running) {
        running = true;
        runTick();
      }
    },
    pause: () => {
      running = false;
      element.classList.add('paused');
    },
  };
}

function progressNow(tl: TimelineInfo): number {
  const now = Number(new Date());
  const passed = (now - tl.ts) * tl.rate;

  return clamp((tl.progressMs + passed) / tl.durationMs, 0, 1);
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(Math.min(value, max), min);
}
