export function animateOnChange(
  el: HTMLElement,
  updated: string,
  resetAnimations: () => void,
  keyframes: Keyframe[] | PropertyIndexedKeyframes | null,
  options?: number | KeyframeAnimationOptions,
) {
  if (el.textContent !== updated) {
    resetAnimations();
    el.textContent = updated;
    el.animate(keyframes, options);
  }
}

export const TextChangeAnimation: [PropertyIndexedKeyframes, KeyframeAnimationOptions] = [
  {
    transform: ['scale(1.1)', 'scale(1)'],
  },
  { duration: 150, easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)' },
];
