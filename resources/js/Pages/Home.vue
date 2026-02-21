<script lang="ts" setup>
import { Head } from '@inertiajs/vue3';
import { computed, onUnmounted, ref, watch } from 'vue';

type CharacterPreview = {
    id: number;
    name: string;
    portrait: string;
};

const props = defineProps<{
    max_visible_killmails?: number;
    max_killmail_age_hours?: number;
    discord_invite_url?: string | null;
}>();

const maxVisibleKillmails = computed(() => {
    return Math.max(1, Number(props.max_visible_killmails ?? 50));
});

const maxKillmailAgeHours = computed(() => {
    return Math.max(1, Number(props.max_killmail_age_hours ?? 168));
});

const discordInviteUrl = computed(() => props.discord_invite_url?.trim() ?? '');

const id = ref('');
const count = ref(maxVisibleKillmails.value);
const hours = ref(maxKillmailAgeHours.value);
const summary = ref(false);

const character = ref<CharacterPreview | null>(null);
const isLoadingCharacter = ref(false);
const characterError = ref('');
const urlCopied = ref(false);
const iskRecipientCopied = ref(false);
const previewRevision = ref(0);
let debounceTimer: number | undefined;
let copyStateTimer: number | undefined;
let iskCopyStateTimer: number | undefined;

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
        hours: hours.value.toString(),
    }).toString();
});

const previewPath = computed(() => {
    if (safeId.value.length === 0) {
        return '';
    }

    return `${origin.value}/${safeId.value}?${query.value}`;
});

const previewFrameSrc = computed(() => {
    if (previewPath.value.length === 0) {
        return '';
    }

    return `${previewPath.value}&preview=${previewRevision.value}`;
});

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

const copyUrl = async () => {
    const copied = await copyText(previewPath.value);

    if (!copied) {
        return;
    }

    urlCopied.value = true;

    if (copyStateTimer !== undefined) {
        clearTimeout(copyStateTimer);
    }

    copyStateTimer = window.setTimeout(() => {
        urlCopied.value = false;
    }, 1600);
};

const copyIskRecipient = async () => {
    const copied = await copyText('Nicolas Kion');

    if (!copied) {
        return;
    }

    iskRecipientCopied.value = true;

    if (iskCopyStateTimer !== undefined) {
        clearTimeout(iskCopyStateTimer);
    }

    iskCopyStateTimer = window.setTimeout(() => {
        iskRecipientCopied.value = false;
    }, 1600);
};

watch(id, (value) => {
    if (value !== safeId.value) {
        id.value = safeId.value;
    }
});

watch([count, hours], () => {
    count.value = Math.max(1, Math.min(count.value, maxVisibleKillmails.value));
    hours.value = Math.max(1, Math.min(hours.value, maxKillmailAgeHours.value));
});

watch([safeId, count, hours, summary], () => {
    previewRevision.value += 1;
});

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

onUnmounted(() => {
    if (copyStateTimer !== undefined) {
        clearTimeout(copyStateTimer);
    }

    if (iskCopyStateTimer !== undefined) {
        clearTimeout(iskCopyStateTimer);
    }
});
</script>

<template>
    <Head title="Setup" />

    <main class="min-h-screen bg-neutral-950 px-4 py-8 text-neutral-100 sm:px-8">
        <div class="mx-auto max-w-6xl border border-neutral-800 bg-neutral-900">
            <section class="border-b border-neutral-800 px-6 py-6 sm:px-8">
                <p class="text-xs tracking-[0.2em] text-neutral-400 uppercase">KillFeed</p>
                <h1 class="mt-2 text-3xl font-semibold tracking-tight">Stream Overlay Setup</h1>
                <p class="mt-3 max-w-3xl text-sm text-neutral-300">
                    Configure your character feed and copy a production-ready browser source URL for OBS. The preview section below renders the exact
                    overlay endpoint you are about to use.
                </p>
            </section>

            <section class="grid gap-0 lg:grid-cols-2">
                <div class="border-b border-neutral-800 p-6 sm:p-8 lg:border-r lg:border-b-0">
                    <label class="block text-sm font-medium text-neutral-200"> Character ID </label>
                    <input
                        v-model="id"
                        class="mt-2 w-full border border-neutral-700 bg-neutral-950 px-3 py-2 text-neutral-100 outline-none focus:border-neutral-500"
                        inputmode="numeric"
                        placeholder="2112625428"
                        type="text"
                    />
                    <p class="mt-3 text-sm leading-relaxed text-neutral-400">
                        The character ID identifies which pilot the backend should track. KillFeed resolves this ID against ESI, stores the character
                        locally, and uses it to query and render all killmail data for both the feed cards and summary values.
                    </p>

                    <div class="mt-6 border border-neutral-800 bg-neutral-950 p-4">
                        <p class="text-xs tracking-wide text-neutral-500 uppercase">Character status</p>
                        <p v-if="isLoadingCharacter" class="mt-2 text-sm text-neutral-300">Looking up character...</p>
                        <p v-else-if="characterError" class="mt-2 text-sm text-red-300">
                            {{ characterError }}
                        </p>
                        <div v-else-if="character" class="mt-3 flex items-center gap-3">
                            <img :alt="character.name" :src="character.portrait" class="size-14 border border-neutral-700" />
                            <div>
                                <p class="font-semibold text-neutral-100">
                                    {{ character.name }}
                                </p>
                                <p class="text-sm text-neutral-400">ID: {{ character.id }}</p>
                            </div>
                        </div>
                        <p v-else class="mt-2 text-sm text-neutral-500">Enter a character ID to begin.</p>
                    </div>
                </div>

                <div class="p-6 sm:p-8">
                    <div>
                        <label class="block text-sm font-medium text-neutral-200"> Visible killmails (maximum) </label>
                        <input
                            v-model.number="count"
                            class="mt-2 w-full border border-neutral-700 bg-neutral-950 px-3 py-2 text-neutral-100 outline-none focus:border-neutral-500"
                            :max="maxVisibleKillmails"
                            min="1"
                            type="number"
                        />
                        <p class="mt-3 text-sm leading-relaxed text-neutral-400">
                            This setting only caps how many killmail cards are displayed at once. If more killmails exist in the selected time window,
                            the overlay shows an overflow tile (+N) so the feed remains compact without hiding that additional events happened.
                            Maximum allowed is
                            {{ maxVisibleKillmails }}.
                        </p>
                    </div>

                    <div class="mt-6">
                        <label class="block text-sm font-medium text-neutral-200"> Maximum killmail age (hours) </label>
                        <input
                            v-model.number="hours"
                            class="mt-2 w-full border border-neutral-700 bg-neutral-950 px-3 py-2 text-neutral-100 outline-none focus:border-neutral-500"
                            :max="maxKillmailAgeHours"
                            min="1"
                            type="number"
                        />
                        <p class="mt-3 text-sm leading-relaxed text-neutral-400">
                            This defines the backend time window for all data. Both the displayed killmails and all summary totals are filtered to
                            events newer than this value. The hard maximum is {{ maxKillmailAgeHours }} hours.
                        </p>
                    </div>

                    <div class="mt-6 border border-neutral-800 bg-neutral-950 p-4">
                        <label class="flex items-center gap-3">
                            <input v-model="summary" class="border border-neutral-600 bg-neutral-950" type="checkbox" />
                            <span class="text-sm font-medium text-neutral-200"> Include summary panel </span>
                        </label>
                        <p class="mt-3 text-sm leading-relaxed text-neutral-400">
                            Enables the right-side summary panel with ISK destroyed, ISK lost, and efficiency. This only changes layout and metrics
                            visibility; it does not override your selected maximum visible killmail count.
                        </p>
                    </div>
                </div>
            </section>

            <section class="border-t border-neutral-800 px-6 py-6 sm:px-8">
                <div class="grid gap-3 md:grid-cols-[1fr_auto] md:items-end">
                    <div class="min-w-0">
                        <p class="text-xs tracking-wide text-neutral-500 uppercase">OBS browser source URL</p>
                        <code
                            class="mt-2 flex h-10 items-center overflow-x-auto border border-neutral-800 bg-neutral-950 px-3 text-sm whitespace-nowrap text-neutral-200"
                        >
                            {{ previewPath || 'Waiting for a valid character ID...' }}
                        </code>
                    </div>
                    <div class="md:self-end">
                        <button
                            :disabled="!canPreview"
                            class="h-10 cursor-pointer border border-neutral-700 bg-neutral-950 px-4 text-sm text-neutral-200 transition-colors hover:bg-neutral-800 disabled:cursor-not-allowed disabled:opacity-40"
                            type="button"
                            @click="copyUrl"
                        >
                            Copy URL
                        </button>
                    </div>
                </div>
                <span v-if="urlCopied" class="mt-2 block text-xs font-medium text-green-400"> URL copied to clipboard </span>
            </section>

            <section class="border-t border-neutral-800 bg-neutral-900 px-6 py-6 sm:px-8">
                <p class="text-xs tracking-wide text-neutral-500 uppercase">About and Support</p>
                <h2 class="mt-2 text-xl font-semibold text-neutral-100">Please Show Your Support</h2>
                <p class="mt-3 max-w-4xl text-sm leading-relaxed text-neutral-300">
                    If KillFeed helps your stream, you can support my projects and my work directly. Check out Mutamarket and Wormhole Systems,
                    support on Patreon, join the community Discord, or send in-game ISK to Nicolas Kion.
                </p>
                <p class="mt-2 max-w-4xl text-sm leading-relaxed text-neutral-400">
                    If you stream, I would appreciate a small chat command that links to one of these pages or an occasional shoutout. Totally
                    optional, but a very nice gesture.
                </p>

                <a
                    v-if="discordInviteUrl"
                    :href="discordInviteUrl"
                    class="mt-4 flex items-center justify-between border border-indigo-500 bg-indigo-500/10 px-4 py-3 text-sm font-semibold text-indigo-200 transition-colors hover:bg-indigo-500/20"
                    rel="noopener noreferrer"
                    target="_blank"
                >
                    <span>Join Community Discord</span>
                    <span aria-hidden="true">-></span>
                </a>

                <div class="mt-4 grid gap-3 sm:grid-cols-2">
                    <a
                        class="border border-neutral-800 bg-neutral-950 px-3 py-3 text-sm text-neutral-200 transition-colors hover:bg-neutral-800"
                        href="https://mutamarket.com"
                        rel="noopener noreferrer"
                        target="_blank"
                    >
                        mutamarket.com
                    </a>
                    <a
                        class="border border-neutral-800 bg-neutral-950 px-3 py-3 text-sm text-neutral-200 transition-colors hover:bg-neutral-800"
                        href="https://wormhole.systems"
                        rel="noopener noreferrer"
                        target="_blank"
                    >
                        wormhole.systems
                    </a>
                    <a
                        class="border border-neutral-800 bg-neutral-950 px-3 py-3 text-sm text-neutral-200 transition-colors hover:bg-neutral-800"
                        href="https://patreon.com/mutamarket"
                        rel="noopener noreferrer"
                        target="_blank"
                    >
                        Patreon: Mutamarket
                    </a>
                    <button
                        class="cursor-pointer border border-neutral-800 bg-neutral-950 px-3 py-3 text-left text-sm text-neutral-200 transition-colors hover:bg-neutral-800"
                        type="button"
                        @click="copyIskRecipient"
                    >
                        In-game ISK: Nicolas Kion
                    </button>
                </div>
                <span v-if="iskRecipientCopied" class="mt-2 block text-xs font-medium text-green-400"> Character name copied to clipboard </span>
            </section>

            <section class="border-t border-neutral-800 px-6 py-6 sm:px-8">
                <p class="text-xs tracking-wide text-neutral-500 uppercase">Live preview</p>
                <div class="mt-3 border border-neutral-800 bg-black p-3">
                    <iframe
                        v-if="canPreview"
                        :key="previewFrameSrc"
                        :src="previewFrameSrc"
                        class="min-h-[166px] w-full border border-neutral-800 bg-neutral-950"
                        loading="eager"
                        scrolling="no"
                        title="KillFeed preview"
                    ></iframe>
                    <div v-else class="grid min-h-[166px] w-full place-items-center border border-dashed border-neutral-700 text-sm text-neutral-500">
                        Waiting for a valid character to render preview.
                    </div>
                </div>
            </section>
        </div>
    </main>
</template>
