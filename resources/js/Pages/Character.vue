<script lang="ts" setup>
import {Head, router} from '@inertiajs/vue3';
import {computed, onMounted, onUnmounted} from "vue";
import {differenceInHours} from "date-fns";

const props = defineProps<{
    character: {
        id: number;
        name: string;
    },
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
}>();

const intl = new Intl.NumberFormat('en-US', {
    notation: 'compact',
    compactDisplay: 'short',
});

let interval: number;

const placeholders = computed(() => {
    return Array.from({length: props.count - props.killmails.length});
});

onMounted(() => {
    interval = setInterval(() => {
        router.reload();
    }, 10000);
});

onUnmounted(() => {
    clearInterval(interval);
});

function isOlderThanHours(killmail: { time: string }, hours: number) {
    let diff = differenceInHours(new Date(), new Date(killmail.time));

    return diff > hours && hours > 0;
}
</script>

<template>
    <Head :title="character.name"/>
    <div

        :style="{ gridTemplateColumns: 'repeat(' + count + ', 1fr)' }"
        class="gap inline-grid overflow-x-hidden divide-neutral-800 divide-x text-white border border-neutral-800 rounded-xl overflow-hidden bg-neutral-900"
    >
        <div
            v-for="killmail in killmails"
            class="">
            <div
                :style="{ backgroundImage: `url(https://images.evetech.net/types/${killmail.ship_type_id}/render?size=128)` }"
                class="size-32"
            >

            </div>
            <div class="grid grid-cols-[auto,1fr,auto] items-center gap-2 px-2 text-lg p-1">
                <div :class="{
                    'bg-red-500': killmail.character_id === character.id,
                    'bg-green-500': killmail.character_id !== character.id,
                }"
                     class="size-2 rounded-full"
                ></div>
                <p class="font-bold mr-auto">{{ intl.format(killmail.value) }}</p>
                <span
                    v-if="killmail.solo"
                    class="text-neutral-500">solo</span>
                <span
                    v-else
                    class="text-neutral-500">{{ killmail.killmail_involvements.length }}</span>
            </div>
        </div>
        <div
            v-for="i in placeholders"
            v-if="killmails.length < count"
            class="">
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
            <div class="border-t border-gray-800 flex justify-center items-center p-1">
                <p class="text-neutral-500 font-bold">N/A</p>
            </div>
        </div>
    </div>
</template>
