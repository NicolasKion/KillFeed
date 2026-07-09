<script lang="ts" setup>
import KillfeedOverlay from '@/Components/KillfeedOverlay.vue';
import type { SimShip } from '@/lib/killfeed';
import { useSimulatedKillfeed } from '@/lib/useSimulatedKillfeed';
import { Head } from '@inertiajs/vue3';
import { onMounted } from 'vue';

const props = defineProps<{
    count: number;
    hours: number;
    summary: boolean;
    meta: boolean;
    sim_ships: SimShip[];
}>();

const { character, killmails, killsValue, lossesValue, killmailsCount, seed, start } = useSimulatedKillfeed({
    maxKillmails: props.count,
    ships: props.sim_ships,
});

onMounted(() => {
    seed(3);
    start();
});
</script>

<template>
    <Head title="Overlay demo">
        <meta name="robots" content="noindex" />
    </Head>

    <KillfeedOverlay
        :character="character"
        :killmails="killmails"
        :hours="hours"
        :summary="summary"
        :show-meta="meta"
        :kills-value="killsValue"
        :losses-value="lossesValue"
        :killmails-count="killmailsCount"
    />
</template>

<style>
html,
body,
#app {
    height: 100%;
    margin: 0;
    overflow: hidden;
}
</style>
