<script lang="ts" setup>
import { Head, router } from '@inertiajs/vue3';
import { computed, onMounted, onUnmounted, ref } from 'vue';

const props = defineProps<{
    character: {
        id: number;
        name: string;
    };
    killmails: {
        id: number;
        hash: string;
        character_id: number;
        corporation_id: number;
        alliance_id: number;
        ship_type_id: number;
        time: string;
        value: number;
        solo: boolean;
        killmail_involvements: {
            character_id: number;
            corporation_id: number;
            alliance_id: number;
            damage_done: number;
            final_blow: boolean;
            security_status: number;
            ship_type_id: number;
            weapon_type_id: number;
        }[];
    }[];
    count: number;
    hours: number;
    summary: boolean;
    kills_value: number;
    losses_value: number;
    killmails_count: number;
}>();

const intl = new Intl.NumberFormat('en-US', {
    notation: 'compact',
    compactDisplay: 'short',
});

const CARD_WIDTH = 128;
const stripRef = ref<HTMLElement | null>(null);
const stripWidth = ref(0);
let interval: number;
let resizeObserver: ResizeObserver | null = null;

const visibleSlots = computed(() => {
    if (stripWidth.value <= 0) {
        return 0;
    }

    return Math.max(Math.floor(stripWidth.value / CARD_WIDTH), 0);
});

const hasOverflow = computed(() => props.killmails_count > visibleSlots.value);

const displayed_killmails = computed(() => {
    const slots = visibleSlots.value;

    if (slots <= 0) {
        return [];
    }

    if (!hasOverflow.value) {
        return props.killmails.slice(0, slots);
    }

    return props.killmails.slice(0, slots);
});

const hiddenKillmails = computed(() => {
    return Math.max(props.killmails_count - visibleSlots.value, 0);
});

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

const isk_efficiency = computed(() => {
    if (props.losses_value === 0) {
        return 100;
    }

    return (props.kills_value / (props.kills_value + props.losses_value)) * 100;
});

onMounted(() => {
    interval = setInterval(() => {
        router.reload();
    }, 10000);

    if (stripRef.value) {
        stripWidth.value = stripRef.value.clientWidth;

        resizeObserver = new ResizeObserver((entries) => {
            stripWidth.value = entries[0]?.contentRect.width ?? 0;
        });

        resizeObserver.observe(stripRef.value);
    }
});

onUnmounted(() => {
    clearInterval(interval);
    resizeObserver?.disconnect();
});
</script>

<template>
    <Head :title="character.name" />

    <div class="w-full max-w-full overflow-hidden border border-neutral-800 bg-neutral-900 text-white">
        <div
            :style="{
                gridTemplateColumns: summary ? 'minmax(0,1fr) minmax(320px,700px)' : 'minmax(0,1fr)',
            }"
            class="grid w-full"
        >
            <div ref="stripRef" class="relative min-w-0 overflow-hidden">
                <TransitionGroup name="list" tag="div" class="flex">
                    <div v-for="killmail in displayed_killmails" :key="killmail.id" class="w-32 shrink-0 border-r border-neutral-800">
                        <div
                            :style="{
                                backgroundImage: `url(https://images.evetech.net/types/${killmail.ship_type_id}/render?size=128)`,
                            }"
                            class="aspect-square w-full bg-cover bg-center"
                        ></div>
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
                                {{ intl.format(kills_value) }}
                            </p>
                        </div>
                        <div class="p-2">
                            <h1 class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500">
                                <span class="size-2 rounded-full bg-red-500"></span>
                                ISK lost
                            </h1>
                            <p class="mb-4 text-right text-5xl font-extrabold text-white">
                                {{ intl.format(losses_value) }}
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
html,
body,
#app {
    height: 100%;
    margin: 0;
    overflow: hidden;
}

.list-move,
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateY(30px);
}

.list-leave-active {
    position: absolute;
}
</style>
