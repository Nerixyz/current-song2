import { State } from './state';
import { MarqueeEl, MarqueeOptions, wrapMarquee } from './text/marquee';

export interface UserScript {
  onPlay(state: State): void;

  onPause(): void;
}

const NOOP = () => undefined;

export function startUserScript(): UserScript {
  const script: UserScript = { onPlay: NOOP, onPause: NOOP };

  // This evaluates to '/user.js'
  // This hack is done to prevent Parcel from being smart and
  // Doing funky stuff with this import like importing it
  // at the start.
  const scriptPath = `/us${(0xe).toString(16)}r.js`;
  try {
    import(scriptPath).then(fns => {
      if (typeof fns !== 'object') return;

      if (typeof fns.onPlay === 'function') {
        script.onPlay = state => {
          try {
            fns.onPlay(state);
          } catch (e) {
            console.warn('Error in user script', e);
          }
        };
      }
      if (typeof fns.onPause === 'function') {
        script.onPause = () => {
          try {
            fns.onPause();
          } catch (e) {
            console.warn('Error in user script', e);
          }
        };
      }
      if (typeof fns.default === 'function') {
        fns.default();
      }
    });
  } catch (e) {
    console.warn('Failed to load user-script', e);
  }

  return script;
}

export interface MarqueeLib {
  wrap: (el: HTMLElement, opts?: MarqueeOptions) => MarqueeEl;
}

export interface CSO2 {
  marquee: MarqueeLib;
}

export function makeCso2Env(): CSO2 {
  return {
    marquee: {
      wrap: wrapMarquee,
    },
  };
}
