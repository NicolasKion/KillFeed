<script lang="ts" setup>
import KillfeedOverlay from '@/Components/KillfeedOverlay.vue';
import { eveImageUrl, type SimShip } from '@/lib/killfeed';
import { useSimulatedKillfeed } from '@/lib/useSimulatedKillfeed';
import { Head } from '@inertiajs/vue3';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import mutamarketLogo from '../../images/support/mutamarket.svg';
import patreonLogo from '../../images/support/patreon.png';
import wormholeLogo from '../../images/support/wormhole-dark.png';

type CharacterPreview = {
    id: number;
    name: string;
    portrait: string;
};

const props = defineProps<{
    max_visible_killmails: number;
    max_killmail_age_hours: number;
    discord_invite_url?: string | null;
    sim_ships: SimShip[];
}>();

const maxVisibleKillmails = computed(() => Math.max(1, props.max_visible_killmails));

const maxKillmailAgeHours = computed(() => Math.max(1, props.max_killmail_age_hours));

const discordInviteUrl = computed(() => props.discord_invite_url?.trim() ?? '');

const id = ref('');
const count = ref(maxVisibleKillmails.value);
const hours = ref(maxKillmailAgeHours.value);
const summary = ref(true);
const meta = ref(true);

const character = ref<CharacterPreview | null>(null);
const isLoadingCharacter = ref(false);
const characterError = ref('');
const copied = ref<'url' | 'isk' | 'chat' | null>(null);
let debounceTimer: number | undefined;
let copiedTimer: number | undefined;
let frameTimer: number | undefined;

const simulation = useSimulatedKillfeed({ ships: props.sim_ships });

const origin = computed(() => {
    if (typeof window === 'undefined') {
        return '';
    }

    return window.location.origin;
});

const safeId = computed(() => id.value.replace(/\D/g, ''));

const query = computed(() => {
    return new URLSearchParams({
        count: count.value.toString(),
        summary: summary.value ? '1' : '0',
        meta: meta.value ? '1' : '0',
        hours: hours.value.toString(),
    }).toString();
});

const previewPath = computed(() => {
    if (safeId.value.length === 0) {
        return '';
    }

    return `${origin.value}/${safeId.value}?${query.value}`;
});

// The iframe follows the options with a debounce so typing in the count
// or hours inputs does not reload the overlay per keystroke. Watchers
// only fire on client-side changes, so this never runs during SSR.
const previewFrameSrc = ref('');

watch(previewPath, (value) => {
    clearTimeout(frameTimer);
    frameTimer = window.setTimeout(() => {
        previewFrameSrc.value = value;
    }, 350);
});

const demoPath = computed(() => `/demo?${query.value}`);

const canPreview = computed(() => character.value !== null && previewPath.value.length > 0);

const copyText = async (value: string) => {
    if (typeof navigator === 'undefined' || !navigator.clipboard || value.length === 0) {
        return false;
    }

    try {
        await navigator.clipboard.writeText(value);
        return true;
    } catch {
        return false;
    }
};

const copyWithFeedback = async (key: 'url' | 'isk' | 'chat', value: string) => {
    if (!(await copyText(value))) {
        return;
    }

    copied.value = key;

    clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => {
        copied.value = null;
    }, 1600);
};

const chatCommandMessage = computed(() => {
    return `Want a killfeed like this on your stream? Build your own at ${origin.value || 'this page'}`;
});

watch(id, (value) => {
    if (value !== safeId.value) {
        id.value = safeId.value;
    }
});

watch([count, hours], () => {
    count.value = Math.max(1, Math.min(count.value, maxVisibleKillmails.value));
    hours.value = Math.max(1, Math.min(hours.value, maxKillmailAgeHours.value));
});

// Pause the simulation while the real feed is on screen.
watch(canPreview, (live) => (live ? simulation.stop() : simulation.start()));

watch(safeId, (value) => {
    character.value = null;
    characterError.value = '';

    if (debounceTimer !== undefined) {
        clearTimeout(debounceTimer);
    }

    if (value.length === 0) {
        isLoadingCharacter.value = false;
        return;
    }

    isLoadingCharacter.value = true;

    debounceTimer = window.setTimeout(async () => {
        try {
            const response = await fetch(`/characters/${value}/preview`, {
                headers: {
                    Accept: 'application/json',
                },
            });

            if (!response.ok) {
                throw new Error('Character not found');
            }

            character.value = (await response.json()) as CharacterPreview;
        } catch {
            characterError.value = 'Character not found. Check the character ID from zKillboard and try again.';
        } finally {
            isLoadingCharacter.value = false;
        }
    }, 350);
});

onMounted(() => {
    simulation.seed(2);
    simulation.start();
});

onUnmounted(() => {
    clearTimeout(copiedTimer);
    clearTimeout(frameTimer);
});
</script>

<template>
    <Head title="EVE Online Stream Overlay" />

    <main class="min-h-screen bg-void font-sans text-starlight">
        <div class="mx-auto px-4 sm:px-10">
            <!-- Top bar -->
            <header class="flex items-center justify-between border-b border-plate py-5">
                <div class="flex items-center gap-4">
                    <svg aria-hidden="true" class="size-5 text-signal" fill="none" viewBox="0 0 64 64">
                        <path d="M8 20V8h12" stroke="currentColor" stroke-width="7" />
                        <path d="M44 8h12v12" stroke="currentColor" stroke-width="7" />
                        <path d="M56 44v12H44" stroke="currentColor" stroke-width="7" />
                        <path d="M20 56H8V44" stroke="currentColor" stroke-width="7" />
                        <circle cx="32" cy="32" r="11" fill="currentColor" />
                    </svg>
                    <p class="font-display text-lg font-bold tracking-[0.25em] uppercase">Kill<span class="text-signal">Feed</span></p>
                    <p class="hidden text-sm text-drift sm:block">EVE Online kill overlay for streamers</p>
                </div>
                <a
                    v-if="discordInviteUrl"
                    :href="discordInviteUrl"
                    class="font-display text-xs font-semibold tracking-[0.15em] text-drift uppercase transition-colors hover:text-starlight"
                    rel="noopener noreferrer"
                    target="_blank"
                >
                    Discord
                </a>
            </header>

            <!-- Hero: the overlay itself, live -->
            <section class="py-10 sm:py-14">
                <h1 class="max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-5xl">Put your kills on stream.</h1>
                <p class="mt-4 max-w-2xl text-base leading-relaxed text-drift">
                    KillFeed turns a character's live killmails into an OBS browser source. The feed below is a simulation — enter your character ID
                    and it switches to the real thing.
                </p>

                <!-- Target-lock frame -->
                <div class="relative mt-10">
                    <span aria-hidden="true" class="absolute -top-px -left-px size-4 border-t-2 border-l-2 border-signal"></span>
                    <span aria-hidden="true" class="absolute -top-px -right-px size-4 border-t-2 border-r-2 border-signal"></span>
                    <span aria-hidden="true" class="absolute -bottom-px -left-px size-4 border-b-2 border-l-2 border-signal"></span>
                    <span aria-hidden="true" class="absolute -right-px -bottom-px size-4 border-r-2 border-b-2 border-signal"></span>

                    <div class="bg-black p-3">
                        <iframe
                            v-if="canPreview"
                            :key="previewFrameSrc"
                            :src="previewFrameSrc"
                            class="min-h-[166px] w-full"
                            loading="eager"
                            scrolling="no"
                            title="KillFeed preview"
                        ></iframe>
                        <KillfeedOverlay
                            v-else
                            :character="simulation.character"
                            :hours="hours"
                            :killmails="simulation.killmails.value"
                            :killmails-count="simulation.killmailsCount.value"
                            :kills-value="simulation.killsValue.value"
                            :losses-value="simulation.lossesValue.value"
                            :show-meta="meta"
                            :summary="summary"
                        />
                    </div>
                </div>

                <!-- Console status line -->
                <div class="mt-3 flex flex-wrap items-center justify-between gap-3 font-mono text-xs">
                    <p v-if="canPreview" class="flex items-center gap-2 text-drift">
                        <span class="size-2 rounded-full bg-confirm motion-safe:animate-pulse"></span>
                        <span class="font-semibold text-confirm">LIVE</span>
                        tracking {{ character?.name }} · {{ character?.id }}
                    </p>
                    <p v-else class="flex items-center gap-2 text-drift">
                        <span class="size-2 rounded-full bg-signal motion-safe:animate-pulse"></span>
                        <span class="font-semibold text-signal">SIM</span>
                        synthetic killmails · a new kill lands every few seconds
                    </p>
                    <a :href="demoPath" class="text-drift underline decoration-plate underline-offset-4 transition-colors hover:text-starlight">
                        open full-page demo
                    </a>
                </div>
            </section>

            <!-- Setup: a real sequence -->
            <section class="grid border-t border-plate lg:grid-cols-3">
                <!-- 01 Track -->
                <div class="border-b border-plate py-8 lg:border-r lg:border-b-0 lg:pr-8">
                    <p class="font-display text-xs font-semibold tracking-[0.2em] text-drift uppercase">
                        <span class="text-signal">01</span>
                        Track
                    </p>

                    <label class="mt-6 block text-sm font-medium" for="character-id">Character ID</label>
                    <input
                        id="character-id"
                        v-model="id"
                        class="mt-2 w-full border-plate bg-hull px-3 py-2 font-mono text-sm text-starlight focus:border-drift focus:ring-0"
                        inputmode="numeric"
                        name="character-id"
                        placeholder="2112625428"
                        type="text"
                    />
                    <p class="mt-3 text-sm leading-relaxed text-drift">
                        Find it in any zKillboard URL: <span class="font-mono text-xs">zkillboard.com/character/&lt;id&gt;</span>
                    </p>

                    <div class="mt-6 border border-plate bg-hull p-4">
                        <p class="font-mono text-xs text-drift">character status</p>
                        <p v-if="isLoadingCharacter" class="mt-2 text-sm text-drift">Looking up character...</p>
                        <p v-else-if="characterError" class="mt-2 text-sm text-signal">
                            {{ characterError }}
                        </p>
                        <div v-else-if="character" class="mt-3 flex items-center gap-3">
                            <img :alt="character.name" :src="character.portrait" class="size-12 border border-plate" />
                            <div>
                                <p class="text-sm font-semibold">{{ character.name }}</p>
                                <p class="font-mono text-xs text-drift">{{ character.id }}</p>
                            </div>
                        </div>
                        <p v-else class="mt-2 text-sm text-drift">Enter a character ID to begin.</p>
                    </div>
                </div>

                <!-- 02 Configure -->
                <div class="border-b border-plate py-8 lg:border-r lg:border-b-0 lg:px-8">
                    <p class="font-display text-xs font-semibold tracking-[0.2em] text-drift uppercase">
                        <span class="text-signal">02</span>
                        Configure
                    </p>

                    <label class="mt-6 block text-sm font-medium" for="visible-killmails">Visible killmails</label>
                    <input
                        id="visible-killmails"
                        v-model.number="count"
                        class="mt-2 w-full border-plate bg-hull px-3 py-2 font-mono text-sm text-starlight focus:border-drift focus:ring-0"
                        :max="maxVisibleKillmails"
                        min="1"
                        name="visible-killmails"
                        type="number"
                    />
                    <p class="mt-2 text-sm leading-relaxed text-drift">
                        Caps the cards on screen; extra kills collapse into a +N tile. Maximum {{ maxVisibleKillmails }}.
                    </p>

                    <label class="mt-6 block text-sm font-medium" for="killmail-age">Killmail age (hours)</label>
                    <input
                        id="killmail-age"
                        v-model.number="hours"
                        class="mt-2 w-full border-plate bg-hull px-3 py-2 font-mono text-sm text-starlight focus:border-drift focus:ring-0"
                        :max="maxKillmailAgeHours"
                        min="1"
                        name="killmail-age"
                        type="number"
                    />
                    <p class="mt-2 text-sm leading-relaxed text-drift">
                        Only kills newer than this show up, in cards and totals. Maximum {{ maxKillmailAgeHours }} hours.
                    </p>

                    <label class="mt-6 flex items-center gap-3">
                        <input v-model="summary" class="border-plate bg-hull text-signal focus:ring-0 focus:ring-offset-0" type="checkbox" />
                        <span class="text-sm font-medium">Include summary panel</span>
                    </label>
                    <p class="mt-2 text-sm leading-relaxed text-drift">Adds ISK destroyed, ISK lost, and efficiency. Try it on the feed above.</p>

                    <label class="mt-6 flex items-center gap-3">
                        <input v-model="meta" class="border-plate bg-hull text-signal focus:ring-0 focus:ring-offset-0" type="checkbox" />
                        <span class="text-sm font-medium">Show meta group icons</span>
                    </label>
                    <p class="mt-2 text-sm leading-relaxed text-drift">
                        Marks Tech II, Tech III, faction, storyline, and officer hulls with their in-game corner icon.
                    </p>
                </div>

                <!-- 03 Deploy -->
                <div class="py-8 lg:pl-8">
                    <p class="font-display text-xs font-semibold tracking-[0.2em] text-drift uppercase">
                        <span class="text-signal">03</span>
                        Deploy
                    </p>

                    <p class="mt-6 block text-sm font-medium">OBS browser source URL</p>
                    <code
                        class="mt-2 flex min-h-10 items-center overflow-x-auto border border-plate bg-hull px-3 py-2 font-mono text-xs whitespace-nowrap"
                        :class="previewPath ? 'text-starlight' : 'text-drift'"
                    >
                        {{ previewPath || 'waiting for a valid character ID...' }}
                    </code>

                    <button
                        :disabled="!canPreview"
                        class="mt-4 w-full cursor-pointer border border-plate bg-hull px-4 py-2.5 font-display text-xs font-semibold tracking-[0.15em] uppercase transition-colors hover:border-drift disabled:cursor-not-allowed disabled:opacity-40"
                        type="button"
                        @click="copyWithFeedback('url', previewPath)"
                    >
                        {{ copied === 'url' ? 'Copied' : 'Copy URL' }}
                    </button>

                    <p class="mt-4 text-sm leading-relaxed text-drift">
                        Add it as a browser source in OBS and stretch it across the bottom of your scene. The feed updates itself every 10 seconds.
                    </p>
                </div>
            </section>

            <!-- Support -->
            <section class="my-10 border border-plate bg-hull px-6 py-8 sm:px-10 sm:py-10">
                <p class="font-display text-xs font-semibold tracking-[0.2em] text-confirm uppercase">Support the project</p>
                <h2 class="mt-3 text-2xl font-semibold tracking-tight sm:text-3xl">Keep KillFeed flying.</h2>
                <p class="mt-3 max-w-2xl text-sm leading-relaxed text-drift">
                    KillFeed is free to use. If it earns a place in your scene, here is what helps the most.
                </p>

                <div class="mt-8 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                    <a
                        class="group flex items-start gap-4 border border-plate bg-void p-4 transition-colors hover:border-confirm/60 hover:bg-hull"
                        href="https://patreon.com/mutamarket"
                        rel="noopener noreferrer"
                        target="_blank"
                    >
                        <span class="grid size-10 shrink-0 place-items-center overflow-hidden border border-plate bg-hull">
                            <img alt="" class="size-10" :src="patreonLogo" />
                        </span>
                        <span class="min-w-0">
                            <span class="flex items-center gap-1.5 font-medium">
                                Patreon
                                <svg
                                    aria-hidden="true"
                                    class="size-3.5 text-drift transition-colors group-hover:text-starlight"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    viewBox="0 0 24 24"
                                >
                                    <path d="M7 17 17 7M9 7h8v8" />
                                </svg>
                            </span>
                            <span class="mt-1 block text-sm leading-relaxed text-drift">Monthly support for hosting and development.</span>
                        </span>
                    </a>
                    <button
                        class="group flex cursor-pointer items-start gap-4 border border-plate bg-void p-4 text-left transition-colors hover:border-confirm/60 hover:bg-hull"
                        type="button"
                        @click="copyWithFeedback('isk', 'Nicolas Kion')"
                    >
                        <span class="grid size-10 shrink-0 place-items-center overflow-hidden border border-plate bg-hull">
                            <img alt="Nicolas Kion" class="size-10" :src="eveImageUrl('characters', 95569887, 'portrait', 64)" />
                        </span>
                        <span class="min-w-0">
                            <span class="block font-medium" :class="{ 'text-confirm': copied === 'isk' }">
                                {{ copied === 'isk' ? 'Copied to clipboard' : 'In-game ISK' }}
                            </span>
                            <span class="mt-1 block text-sm leading-relaxed text-drift">Send ISK to Nicolas Kion. Click to copy the name.</span>
                        </span>
                    </button>
                    <a
                        class="group flex items-start gap-4 border border-plate bg-void p-4 transition-colors hover:border-confirm/60 hover:bg-hull"
                        href="https://mutamarket.com"
                        rel="noopener noreferrer"
                        target="_blank"
                    >
                        <span class="grid size-10 shrink-0 place-items-center border border-plate bg-hull">
                            <img alt="" class="size-6" :src="mutamarketLogo" />
                        </span>
                        <span class="min-w-0">
                            <span class="flex items-center gap-1.5 font-medium">
                                mutamarket.com
                                <svg
                                    aria-hidden="true"
                                    class="size-3.5 text-drift transition-colors group-hover:text-starlight"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    viewBox="0 0 24 24"
                                >
                                    <path d="M7 17 17 7M9 7h8v8" />
                                </svg>
                            </span>
                            <span class="mt-1 block text-sm leading-relaxed text-drift">EVE market tools from the same developer.</span>
                        </span>
                    </a>
                    <a
                        class="group flex items-start gap-4 border border-plate bg-void p-4 transition-colors hover:border-confirm/60 hover:bg-hull"
                        href="https://wormhole.systems"
                        rel="noopener noreferrer"
                        target="_blank"
                    >
                        <span class="grid size-10 shrink-0 place-items-center border border-plate bg-hull">
                            <img alt="" class="size-6" :src="wormholeLogo" />
                        </span>
                        <span class="min-w-0">
                            <span class="flex items-center gap-1.5 font-medium">
                                wormhole.systems
                                <svg
                                    aria-hidden="true"
                                    class="size-3.5 text-drift transition-colors group-hover:text-starlight"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    viewBox="0 0 24 24"
                                >
                                    <path d="M7 17 17 7M9 7h8v8" />
                                </svg>
                            </span>
                            <span class="mt-1 block text-sm leading-relaxed text-drift">Wormhole tooling from the same developer.</span>
                        </span>
                    </a>
                </div>

                <div class="mt-8 border border-plate bg-void p-5">
                    <p class="font-display text-xs font-semibold tracking-[0.2em] text-drift uppercase">Spread it in chat</p>
                    <p class="mt-2 max-w-2xl text-sm leading-relaxed text-drift">
                        Viewers will ask about the overlay mid-fight. A <span class="font-mono text-starlight">!killfeed</span> chat command answers
                        for you and sends them here to set up their own.
                    </p>
                    <div class="mt-3 flex flex-wrap items-stretch gap-3">
                        <code
                            class="flex min-h-10 grow items-center overflow-x-auto border border-plate bg-hull px-3 py-2 font-mono text-xs whitespace-nowrap text-starlight"
                        >
                            {{ chatCommandMessage }}
                        </code>
                        <button
                            class="cursor-pointer border border-plate bg-hull px-4 py-2 font-display text-xs font-semibold tracking-[0.15em] uppercase transition-colors hover:border-drift"
                            :class="{ 'text-confirm': copied === 'chat' }"
                            type="button"
                            @click="copyWithFeedback('chat', chatCommandMessage)"
                        >
                            {{ copied === 'chat' ? 'Copied' : 'Copy message' }}
                        </button>
                    </div>
                </div>
            </section>

            <footer class="border-t border-plate py-6">
                <p class="font-mono text-xs text-drift">
                    KillFeed is a community project. EVE Online and all related assets are the property of CCP hf.
                </p>
            </footer>
        </div>
    </main>
</template>
