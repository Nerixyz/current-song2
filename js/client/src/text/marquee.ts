export interface MarqueeOptions {
  speed: number;
  pauseDuration: number;
  repeatPauseDuration: number;
}

export interface MarqueeEl {
  pause: () => void;
  start: () => void;
  reset: () => void;
}

const defaultOpts: MarqueeOptions = {
  speed: 0.2,
  pauseDuration: 1200,
  repeatPauseDuration: 2000,
};

export function wrapMarquee(el: HTMLElement, opts: MarqueeOptions = defaultOpts): MarqueeEl {
  opts = { ...defaultOpts, ...opts };

  const wrap = make('div', 'mq-wrap');
  wrap.id = `mq-${el.id}`;
  const mask = make('div', 'mq-mask');
  const guard = make('div', 'mq-overflow-guard');
  const userWrap = make('div', 'mq-user-wrap');

  const parent = el.parentElement ?? document.documentElement;
  const sibling = el.nextSibling;
  el.remove();
  parent.insertBefore(wrap, sibling);

  wrap.appendChild(mask);
  mask.appendChild(guard);
  guard.appendChild(userWrap);
  userWrap.appendChild(el);

  const diffWidth = () => userWrap.clientWidth - mask.clientWidth;

  let cbID: null | number = null;
  let lastTime = performance.now();
  let pos = 0;
  let waitFrom = lastTime;
  let dir = 1;
  const applyPos = () => userWrap.style.setProperty('--marquee', `${-pos}px`);

  const actualAnimationFrame = (time: number) => {
    const deltaTime = time - lastTime;
    lastTime = time;
    const overflow = diffWidth();

    if (overflow <= 0) {
      if (pos != 0) {
        pos = 0;
        applyPos();
      }
      return;
    }

    if (waitFrom > 0) {
      if (time > waitFrom + opts.pauseDuration) {
        waitFrom = 0;
      }
    } else {
      pos += dir * ((60 * deltaTime) / 1000) * opts.speed;
      if (pos > overflow) {
        dir *= -1;
        waitFrom = time;
        pos = overflow;
      } else if (pos < 0) {
        dir *= -1;
        waitFrom = time + opts.repeatPauseDuration;
        pos = 0;
      }
    }
    applyPos();
  };
  const onAnimationFrame = (time: number) => {
    actualAnimationFrame(time);
    if (cbID) {
      cbID = requestAnimationFrame(onAnimationFrame);
    }
  };

  return {
    start: () => {
      if (!cbID) {
        lastTime = performance.now();
        cbID = requestAnimationFrame(onAnimationFrame);
      }
    },
    pause: () => {
      if (cbID) {
        cancelAnimationFrame(cbID);
        cbID = null;
      }
    },
    reset: () => {
      dir = 1;
      pos = 0;
      waitFrom = lastTime;
      applyPos();
    },
  };
}

function make(ty: keyof HTMLElementTagNameMap, ...classes: string[]): HTMLElement {
  const el = document.createElement(ty);
  el.classList.add(...classes);
  return el;
}
