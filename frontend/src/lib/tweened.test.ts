import { describe, expect, it } from 'vitest';

import { TWEEN_DURATION_MS, easeOutCubic, tweenDuration } from './tweened.ts';

describe('easeOutCubic', () => {
  it('starts fast and lands on the target', () => {
    expect(easeOutCubic(0)).toBe(0);
    expect(easeOutCubic(1)).toBe(1);
    expect(easeOutCubic(0.5)).toBe(0.875);
  });
});

describe('tweenDuration', () => {
  it('eases over 800 ms and snaps for reduced motion', () => {
    expect(TWEEN_DURATION_MS).toBe(800);
    expect(tweenDuration(false)).toBe(800);
    expect(tweenDuration(true)).toBe(0);
  });
});
