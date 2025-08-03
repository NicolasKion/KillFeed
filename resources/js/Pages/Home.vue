<script lang="ts" setup>
import { Head } from "@inertiajs/vue3";
import { computed, ref } from "vue";

defineProps<{}>();

const intl = new Intl.NumberFormat("en-US", {
    notation: "compact",
    compactDisplay: "short",
});

const id = ref("");
const count = ref(5);
const summary = ref(false);

const path = computed(() => {
    const hostname = window.location.hostname;
    const params = new URLSearchParams({
        count: count.value.toString(),
        summary: summary.value ? "1" : "0",
        hours: hours.value.toString(),
    });

    return `https://${hostname}/${id.value}?${params}`;
});

const size = computed(() => {
    const size_kills = count.value * 128;
    const size_summary = 700;
    return `${summary.value ? size_kills + size_summary : size_kills}x${166}`;
});

const hours = ref(24);
</script>

<template>
    <Head title="Setup" />
    <div class="p-8">
        <h1 class="text-4xl mb-8">Setting up KillFeed</h1>
        <p class="my-4">
            Setting up KillFeed is easy. Just follow these steps:
        </p>
        <ol class="grid gap-4">
            <li>
                Enter your character's ID in the field below (You can get it
                from zKillboard by searching for your character and looking at
                the URL)
                <input
                    v-model="id"
                    class="border border-gray-300 rounded-md p-2 w-full mt-2"
                    type="text"
                />
            </li>
            <li>
                Enter the maximum number of killmails you want to display at
                once.
                <input
                    v-model="count"
                    class="border border-gray-300 rounded-md p-2 w-full mt-2"
                    type="number"
                />
            </li>
            <li>
                Filter out killmails older than a certain number of hours. Maximum is 7 days (168 hours).
                <input
                    v-model="hours"
                    class="border border-gray-300 rounded-md p-2 w-full mt-2"
                    type="number"
                    min="1"
                    max="168"
                />
            </li>
            <li>
                <input
                    id="summary"
                    v-model="summary"
                    class="border border-gray-300 rounded-md p-2 mt-2"
                    type="checkbox"
                />
                <label class="ml-2" for="summary">
                    Show a summary of all killmails within the last
                    {{ hours || "24" }} hours (isk destroyed, efficiency, etc.)
                </label>
            </li>
            <li v-if="id">
                Copy the following URL and add it as a browser source in OBS or
                your streaming software.
                <pre class="bg-gray-800 text-white p-2 rounded-md mt-2">{{
                    path
                }}</pre>
            </li>
            <li v-if="id">
                Set the size of the browser source to {{ size }} pixels. Make
                sure that you are actually setting the size of the browser
                source and not just scaling its output.
            </li>
        </ol>
    </div>
</template>
