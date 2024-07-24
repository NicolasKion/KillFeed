<script lang="ts" setup>
import {Head, router} from '@inertiajs/vue3';
import {onMounted, onUnmounted} from "vue";

defineProps<{
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
        killmail_time: string;
        value: number;
        solo: boolean;
        attackers: {
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
}>();

const intl = new Intl.NumberFormat('en-US', {
    notation: 'compact',
    compactDisplay: 'short',
});

let interval: number;

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
    <Head :title="character.name"/>
    <div
        class="gap grid-cols-5 inline-grid divide-neutral-800 divide-x text-white border border-neutral-800 rounded-xl overflow-hidden bg-neutral-900">
        <div
            v-for="killmail in killmails"
            class="">
            <img
                :alt="`Icon for ${killmail.ship_type_id}`"
                :src="`https://images.evetech.net/types/${killmail.ship_type_id}/render?size=128`"
                class="size-32"
            />
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
                    class="text-neutral-500">{{ killmail.attackers.length }}</span>
            </div>
        </div>
    </div>
</template>
