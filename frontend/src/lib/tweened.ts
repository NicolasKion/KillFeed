// The summary numbers ease towards a new value (800 ms, cubic out) and
// snap when the viewer prefers reduced motion.

export const TWEEN_DURATION_MS = 800;

export function easeOutCubic(_t: number): number {
  throw new Error('not implemented');
}

export function tweenDuration(_reducedMotion: boolean): number {
  throw new Error('not implemented');
}
