<script lang="ts" setup>
import { Head, router } from "@inertiajs/vue3";
import { computed, onMounted, onUnmounted } from "vue";

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

const intl = new Intl.NumberFormat("en-US", {
    notation: "compact",
    compactDisplay: "short",
});

let interval: number;

const displayed_killmails = computed(() => {
    if (props.killmails_count > props.count) {
        return props.killmails.slice(0, props.count - 1);
    }

    return props.killmails;
});

const placeholders = computed(() => {
    return Array.from({ length: props.count - props.killmails.length });
});

const isk_efficiency = computed(() => {
    // Killed 100m, lost 50m = 66.67%
    // Killed 50m, lost 100m = 33.33%

    if (props.losses_value === 0) {
        return 100;
    }

    return (props.kills_value / (props.kills_value + props.losses_value)) * 100;
});

onMounted(() => {
    interval = setInterval(() => {
        router.reload();
    }, 10000);
});

onUnmounted(() => {
    clearInterval(interval);
});
</script>

<template>
    <Head :title="character.name" />
    <div
        :class="{
            'bg-neutral-900': summary,
        }"
        class="flex justify-between items-center"
    >
        <div
            :class="{
                'rounded-xl': !summary,
            }"
            :style="{
                gridTemplateColumns: 'repeat(' + count + ', 1fr)',
            }"
            class="gap inline-grid overflow-x-hidden divide-neutral-800 divide-x text-white border border-neutral-800 overflow-hidden bg-neutral-900"
        >
            <div v-for="killmail in displayed_killmails" class="">
                <div
                    :style="{
                        backgroundImage: `url(https://images.evetech.net/types/${killmail.ship_type_id}/render?size=128)`,
                    }"
                    class="size-32"
                ></div>
                <div
                    class="grid grid-cols-[auto,1fr,auto] items-center gap-2 px-2 text-lg p-1"
                >
                    <div
                        :class="{
                            'bg-red-500':
                                killmail.character_id === character.id,
                            'bg-green-500':
                                killmail.character_id !== character.id,
                        }"
                        class="size-2 rounded-full"
                    ></div>
                    <p class="font-bold mr-auto">
                        {{ intl.format(killmail.value) }}
                    </p>
                    <span v-if="killmail.solo" class="text-neutral-500"
                        >solo</span
                    >
                    <span v-else class="text-neutral-500">{{
                        killmail.killmail_involvements.length
                    }}</span>
                </div>
            </div>
            <div
                v-for="i in placeholders"
                v-if="killmails.length < count"
                class=""
            >
                <svg
                    class="size-32"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <line
                        stroke-width="0.01"
                        x1="0"
                        x2="24"
                        y1="0"
                        y2="24"
                    ></line>
                    <line
                        stroke-width="0.01"
                        x1="0"
                        x2="24"
                        y1="24"
                        y2="0"
                    ></line>
                </svg>
                <div
                    class="border-t border-neutral-800 flex justify-center items-center p-1"
                >
                    <p class="text-neutral-500 font-bold">N/A</p>
                </div>
            </div>
            <div
                v-if="killmails_count > count"
                class="text-neutral-500 bg-neutral-900"
            >
                <div
                    class="flex aspect-square justify-center items-center text-4xl font-extrabold"
                >
                    + {{ killmails_count - displayed_killmails.length }}
                </div>
                <div
                    class="flex justify-center items-center py-2 border-t border-neutral-800"
                >
                    <p class="text-center">last {{ hours }} hours</p>
                </div>
            </div>
        </div>
        <div v-if="summary" class="grid grid-cols-3 gap-16 p-2 mr-16">
            <div class="p-2">
                <h1
                    class="text-lg font-bold flex items-center gap-2 text-neutral-500"
                >
                    <div class="size-2 rounded-full bg-green-500"></div>
                    ISK destroyed
                </h1>
                <p class="text-white text-5xl mb-4 font-extrabold text-right">
                    {{ intl.format(kills_value) }}
                </p>
            </div>
            <div class="p-2">
                <h1
                    class="text-lg font-bold flex items-center gap-2 text-neutral-500"
                >
                    <div class="size-2 rounded-full bg-red-500"></div>
                    ISK lost
                </h1>
                <p class="text-white text-5xl mb-4 font-extrabold text-right">
                    {{ intl.format(losses_value) }}
                </p>
            </div>
            <div class="p-2">
                <h1
                    class="text-lg font-bold flex items-center gap-2 text-neutral-500"
                >
                    <div class="size-2 rounded-full bg-amber-500"></div>
                    ISK efficiency
                </h1>
                <p class="text-white text-5xl mb-4 font-extrabold text-right">
                    {{ isk_efficiency.toFixed(0) }}%
                </p>
            </div>
        </div>
    </div>
</template>
