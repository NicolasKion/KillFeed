// The summary numbers ease towards a new value (800 ms, cubic out) and
// snap when the viewer prefers reduced motion.

export const TWEEN_DURATION_MS = 800;

export function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3;
}

export function tweenDuration(reducedMotion: boolean): number {
  return reducedMotion ? 0 : TWEEN_DURATION_MS;
}
