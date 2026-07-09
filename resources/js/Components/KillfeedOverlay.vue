<script lang="ts" setup>
import { eveImageUrl, META_BADGES, type OverlayCharacter, type OverlayKillmail } from '@/lib/killfeed';
import { useTweenedNumber } from '@/lib/useTweenedNumber';
import { computed, onMounted, onUnmounted, ref } from 'vue';

const props = withDefaults(
    defineProps<{
        character: OverlayCharacter;
        killmails: OverlayKillmail[];
        hours: number;
        summary: boolean;
        showMeta?: boolean;
        killsValue: number;
        lossesValue: number;
        killmailsCount: number;
    }>(),
    { showMeta: true },
);

const intl = new Intl.NumberFormat('en-US', {
    notation: 'compact',
    compactDisplay: 'short',
});

const CARD_WIDTH = 128;
const stripRef = ref<HTMLElement | null>(null);
const stripWidth = ref(0);
let resizeObserver: ResizeObserver | null = null;

const visibleSlots = computed(() => {
    if (stripWidth.value <= 0) {
        return 0;
    }

    return Math.max(Math.floor(stripWidth.value / CARD_WIDTH), 0);
});

const displayed_killmails = computed(() => {
    const slots = visibleSlots.value;

    if (slots <= 0) {
        return [];
    }

    return props.killmails.slice(0, slots);
});

const hiddenKillmails = computed(() => {
    return Math.max(props.killmailsCount - visibleSlots.value, 0);
});

const hasOverflow = computed(() => hiddenKillmails.value > 0);

const timeWindowLabel = computed(() => {
    if (props.hours > 24) {
        const days = (props.hours / 24).toFixed(1).replace(/\.0$/, '');

        return `last ${days} days`;
    }

    return `last ${props.hours} hours`;
});

const placeholderSlots = computed(() => {
    return Math.max(visibleSlots.value - displayed_killmails.value.length, 0);
});

const tweenedKills = useTweenedNumber(() => props.killsValue);
const tweenedLosses = useTweenedNumber(() => props.lossesValue);

const isk_efficiency = computed(() => {
    if (tweenedLosses.value === 0) {
        return 100;
    }

    return (tweenedKills.value / (tweenedKills.value + tweenedLosses.value)) * 100;
});

onMounted(() => {
    if (stripRef.value) {
        stripWidth.value = stripRef.value.clientWidth;

        resizeObserver = new ResizeObserver((entries) => {
            stripWidth.value = entries[0]?.contentRect.width ?? 0;
        });

        resizeObserver.observe(stripRef.value);
    }
});

onUnmounted(() => {
    resizeObserver?.disconnect();
});

// Leaving cards go position:absolute so their siblings can slide over; pin
// them to their current slot first or they snap to the strip's origin.
const pinLeavePosition = (element: Element) => {
    const card = element as HTMLElement;

    card.style.left = `${card.offsetLeft}px`;
    card.style.top = `${card.offsetTop}px`;
};
</script>

<template>
    <div class="w-full max-w-full overflow-hidden border border-neutral-800 bg-neutral-900 text-white">
        <div
            :style="{
                gridTemplateColumns: summary ? 'minmax(0,1fr) minmax(320px,700px)' : 'minmax(0,1fr)',
            }"
            class="grid w-full"
        >
            <div ref="stripRef" class="relative min-w-0 overflow-hidden">
                <TransitionGroup name="list" tag="div" class="flex" @before-leave="pinLeavePosition">
                    <div v-for="killmail in displayed_killmails" :key="killmail.id" class="w-32 shrink-0 border-r border-neutral-800">
                        <div
                            :style="{
                                backgroundImage: `url(${eveImageUrl('types', killmail.ship_type_id, 'render', 128)})`,
                            }"
                            class="relative aspect-square w-full bg-cover bg-center"
                        >
                            <img
                                v-if="showMeta && killmail.ship_meta_group_id != null && META_BADGES[killmail.ship_meta_group_id]"
                                :alt="META_BADGES[killmail.ship_meta_group_id].label"
                                :title="META_BADGES[killmail.ship_meta_group_id].label"
                                class="absolute top-0 left-0 size-6"
                                :src="META_BADGES[killmail.ship_meta_group_id].icon"
                            />
                        </div>
                        <div class="flex items-center gap-2 px-2 py-1 text-lg whitespace-nowrap">
                            <div
                                :class="{
                                    'bg-red-500': killmail.character_id === character.id,
                                    'bg-green-500': killmail.character_id !== character.id,
                                }"
                                class="size-2 rounded-full"
                            ></div>
                            <p class="mr-auto font-bold">
                                {{ intl.format(killmail.value) }}
                            </p>
                            <span v-if="killmail.solo" class="text-neutral-500">solo</span>
                            <span v-else class="text-neutral-500">{{ killmail.killmail_involvements.length }}</span>
                        </div>
                    </div>

                    <div
                        v-for="index in placeholderSlots"
                        :key="`placeholder-${index}`"
                        class="w-32 shrink-0 border-r border-neutral-800 text-neutral-500"
                    >
                        <div class="grid aspect-square w-full place-items-center bg-neutral-900">
                            <svg
                                aria-hidden="true"
                                class="h-full w-full text-neutral-800"
                                fill="none"
                                viewBox="0 0 100 100"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <line x1="0" x2="100" y1="0" y2="100" stroke="currentColor" stroke-width="1"></line>
                                <line x1="0" x2="100" y1="100" y2="0" stroke="currentColor" stroke-width="1"></line>
                            </svg>
                        </div>
                        <div class="flex items-center justify-center border-t border-neutral-800 py-1">
                            <p class="font-bold">N/A</p>
                        </div>
                    </div>
                </TransitionGroup>

                <div v-if="hasOverflow" class="absolute inset-y-0 right-0 w-32 border-l border-neutral-800 bg-neutral-900 text-neutral-500">
                    <div class="flex aspect-square items-center justify-center text-4xl font-extrabold">+ {{ hiddenKillmails }}</div>
                    <div class="flex items-center justify-center border-t border-neutral-800 py-2">
                        <p class="text-center">{{ timeWindowLabel }}</p>
                    </div>
                </div>

                <div
                    v-if="hasOverflow"
                    class="pointer-events-none absolute inset-y-0 right-32 w-16 bg-gradient-to-l from-neutral-900 to-transparent"
                ></div>
            </div>

            <div v-if="summary" class="border-l border-neutral-800">
                <div class="relative h-full bg-neutral-900">
                    <div
                        v-if="!hasOverflow"
                        class="pointer-events-none absolute inset-y-0 right-full w-16 bg-gradient-to-l from-neutral-900 to-transparent"
                    ></div>
                    <div class="mx-8 grid h-full grid-cols-3 items-center gap-16 p-2">
                        <div class="p-2">
                            <h1 class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500">
                                <span class="size-2 rounded-full bg-green-500"></span>
                                ISK destroyed
                            </h1>
                            <p class="mb-4 text-right text-5xl font-extrabold text-white">
                                {{ intl.format(tweenedKills) }}
                            </p>
                        </div>
                        <div class="p-2">
                            <h1 class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500">
                                <span class="size-2 rounded-full bg-red-500"></span>
                                ISK lost
                            </h1>
                            <p class="mb-4 text-right text-5xl font-extrabold text-white">
                                {{ intl.format(tweenedLosses) }}
                            </p>
                        </div>
                        <div class="p-2">
                            <h1 class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500">
                                <span class="size-2 rounded-full bg-amber-500"></span>
                                ISK efficiency
                            </h1>
                            <p class="mb-4 text-right text-5xl font-extrabold text-white">{{ isk_efficiency.toFixed(0) }}%</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style>
.list-move,
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

/* New cards slide in from the left, matching the feed's flow direction. */
.list-enter-from {
    opacity: 0;
    transform: translateX(-100%);
}

/* Old cards get pushed out to the right, sliding behind the overflow tile. */
.list-leave-to {
    opacity: 0;
    transform: translateX(100%);
}

.list-leave-active {
    position: absolute;
}
</style>
