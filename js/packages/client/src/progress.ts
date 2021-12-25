import { TimelineInfo } from './types';

export function createProgress(element: HTMLDivElement): {
  run: (timeline: TimelineInfo | null) => void;
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

    const newProgress = progressNow(currentTl);
    if (progress !== newProgress) {
      progress = newProgress;
      element.style.setProperty('--progress', progress.toString());
    }
    requestAnimationFrame(runTick);
  };

  return {
    run: timeline => {
      currentTl = timeline;
      element.classList.remove('hidden');

      if (!running) {
        running = true;
        runTick();
      }
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
