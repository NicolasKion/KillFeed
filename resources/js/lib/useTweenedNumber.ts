import { onUnmounted, ref, watch } from 'vue';

const easeOutCubic = (t: number) => 1 - (1 - t) ** 3;

/**
 * Returns a ref that follows the source value, easing towards it instead of
 * jumping. Respects prefers-reduced-motion by snapping immediately.
 */
export function useTweenedNumber(source: () => number, durationMs = 800) {
    const display = ref(source());
    let frame: number | undefined;

    const prefersReducedMotion = () => typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    watch(source, (target) => {
        if (frame !== undefined) {
            cancelAnimationFrame(frame);
            frame = undefined;
        }

        if (typeof window === 'undefined' || prefersReducedMotion()) {
            display.value = target;
            return;
        }

        const from = display.value;
        const start = performance.now();

        const step = (now: number) => {
            const progress = Math.min((now - start) / durationMs, 1);

            display.value = from + (target - from) * easeOutCubic(progress);

            if (progress < 1) {
                frame = requestAnimationFrame(step);
            } else {
                frame = undefined;
            }
        };

        frame = requestAnimationFrame(step);
    });

    onUnmounted(() => {
        if (frame !== undefined) {
            cancelAnimationFrame(frame);
        }
    });

    return display;
}
