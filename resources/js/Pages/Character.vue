<script lang="ts" setup>
import KillfeedOverlay from '@/Components/KillfeedOverlay.vue';
import type { OverlayCharacter, OverlayKillmail } from '@/lib/killfeed';
import { Head, router } from '@inertiajs/vue3';
import { onMounted, onUnmounted } from 'vue';

defineProps<{
    character: OverlayCharacter;
    killmails: OverlayKillmail[];
    hours: number;
    summary: boolean;
    meta: boolean;
    kills_value: number;
    losses_value: number;
    killmails_count: number;
}>();

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
    <Head :title="character.name">
        <meta name="robots" content="noindex" />
    </Head>

    <KillfeedOverlay
        :character="character"
        :killmails="killmails"
        :hours="hours"
        :summary="summary"
        :show-meta="meta"
        :kills-value="kills_value"
        :losses-value="losses_value"
        :killmails-count="killmails_count"
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
