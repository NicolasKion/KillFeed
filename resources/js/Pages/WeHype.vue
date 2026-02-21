<script lang="ts" setup>
import { router } from '@inertiajs/vue3';
import { formatDuration } from 'date-fns';
import { computed, onMounted, onUnmounted, ref } from 'vue';

type TCreator = {
    id: string;
    externalIdentifier: string;
    backgroundImageUrl: string;
    displayName: string;
    games: {
        game: {
            slug: string;
            name: string;
            coverImage: {
                transformedUrl: string;
            };
            id: string;
        };
        rewardCycle: {
            status: string;
            iHaveJoined: boolean | null;
            rewards: {
                id: string;
                status: string;
                requiredStreamedMinutes: number;
                streamedMinutes: number;
                sku: {
                    image: {
                        transformedUrl: string;
                    };
                };
                myResult: {
                    status: string;
                    rewardedAt: string | null;
                } | null;
            }[];
            id: string;
        };
        viewerMinutes: number;
        streamedMinutes: number;
    }[];
};

const { creator, slug } = defineProps<{ creator: TCreator; slug: string }>();

const eve = computed(() => {
    return creator.games.find((game) => game.game.slug === 'eve-online');
});

const active_reward = computed(() => {
    return eve?.value?.rewardCycle.rewards.filter((reward) => reward.status === 'IN_PROGRESS').at(0);
});

const time_completed = computed(() => {
    const totalMinutes = active_reward.value?.streamedMinutes || 0;
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    return formatDuration({ hours, minutes }, { format: ['hours', 'minutes'], delimiter: ' ' })
        .replace(' hours', 'h')
        .replace(' minutes', 'm');
});

const total_minutes = computed(() => {
    const totalMinutes = active_reward.value?.requiredStreamedMinutes || 0;
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    return formatDuration({ hours, minutes }, { format: ['hours', 'minutes'], delimiter: ' ' })
        .replace(' hours', 'h')
        .replace(' minutes', 'm');
});

const link = computed(() => {
    return `https://${slug}.wehype.app/eveonline`;
});

const percentage = computed(() => {
    return Math.floor(((active_reward.value?.streamedMinutes || 0) / (active_reward.value?.requiredStreamedMinutes || 1)) * 100);
});

const interval = ref();

onMounted(() => {
    interval.value = setInterval(() => {
        router.reload();
    }, 60_000);
});

onUnmounted(() => {
    if (interval.value) clearInterval(interval.value);
});
</script>

<template>
    <div class="grid grid-cols-[auto_1fr] items-center gap-x-4 rounded-lg bg-neutral-900 p-2 px-2 pr-8 text-white">
        <img v-if="active_reward" :src="active_reward.sku.image.transformedUrl" alt="active reward" class="row-span-3 h-20 w-20 rounded-lg" />

        <p v-if="active_reward" class="text-2xl text-white">
            <span>{{ time_completed }}</span>
            /
            <span>{{ total_minutes }}</span>
            streamed
        </p>
        <div class="h-1 overflow-hidden rounded-full bg-neutral-600">
            <div :style="`width: ${percentage}%`" class="h-full rounded-full bg-green-500"></div>
        </div>
        <p class="leading-none text-neutral-400">
            Join now at <b>{{ link }}</b>
        </p>
    </div>
</template>

<style scoped></style>
