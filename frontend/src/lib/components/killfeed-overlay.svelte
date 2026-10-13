<script lang="ts">
  // The overlay: as many 128 pixel killmail cards as the strip fits (none
  // before it is measured, so the server renders an empty strip), N/A
  // tiles for the empty slots, a `+N` tile for what does not fit, and the
  // optional summary panel with eased totals.
  import { untrack } from 'svelte';
  import { flip } from 'svelte/animate';
  import { Tween, prefersReducedMotion } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { fly } from 'svelte/transition';

  import {
    META_BADGES,
    eveImageUrl,
    type OverlayCharacter,
    type OverlayKillmail,
  } from '#lib/killfeed.ts';
  import {
    formatIsk,
    hiddenKillmails,
    iskEfficiency,
    timeWindowLabel,
    visibleSlots,
  } from '#lib/overlay.ts';
  import { tweenDuration } from '#lib/tweened.ts';

  let {
    character,
    killmails,
    hours,
    summary,
    showMeta = true,
    killsValue,
    lossesValue,
    killmailsCount,
  }: {
    character: OverlayCharacter;
    killmails: OverlayKillmail[];
    hours: number;
    summary: boolean;
    showMeta?: boolean;
    killsValue: number;
    lossesValue: number;
    killmailsCount: number;
  } = $props();

  let stripWidth = $state(0);

  const slots = $derived(visibleSlots(stripWidth));
  const displayed = $derived(slots <= 0 ? [] : killmails.slice(0, slots));
  const hidden = $derived(hiddenKillmails(killmailsCount, slots));
  const hasOverflow = $derived(hidden > 0);
  const placeholders = $derived(
    Array.from(
      { length: Math.max(slots - displayed.length, 0) },
      (_, index) => `placeholder-${index}`,
    ),
  );

  const tweenedKills = new Tween(
    untrack(() => killsValue),
    { easing: cubicOut },
  );
  const tweenedLosses = new Tween(
    untrack(() => lossesValue),
    { easing: cubicOut },
  );

  $effect(() => {
    tweenedKills.set(killsValue, { duration: tweenDuration(prefersReducedMotion.current) });
  });
  $effect(() => {
    tweenedLosses.set(lossesValue, { duration: tweenDuration(prefersReducedMotion.current) });
  });

  const efficiency = $derived(iskEfficiency(tweenedKills.current, tweenedLosses.current));
  const motion = { duration: 500, easing: cubicOut };
</script>

<div class="w-full max-w-full overflow-hidden border border-neutral-800 bg-neutral-900 text-white">
  <div
    style:grid-template-columns={summary ? 'minmax(0,1fr) minmax(320px,700px)' : 'minmax(0,1fr)'}
    class="grid w-full"
  >
    <div bind:clientWidth={stripWidth} class="relative min-w-0 overflow-hidden">
      <div class="flex">
        {#each displayed as killmail (killmail.id)}
          <div
            data-slot
            class="w-32 shrink-0 border-r border-neutral-800"
            animate:flip={motion}
            in:fly={{ ...motion, x: -128 }}
          >
            <div
              style:background-image="url({eveImageUrl(
                'types',
                killmail.ship_type_id,
                'render',
                128,
              )})"
              class="relative aspect-square w-full bg-cover bg-center"
            >
              {#if showMeta && killmail.ship_meta_group_id != null && META_BADGES[killmail.ship_meta_group_id]}
                <img
                  alt={META_BADGES[killmail.ship_meta_group_id].label}
                  title={META_BADGES[killmail.ship_meta_group_id].label}
                  class="absolute top-0 left-0 size-6"
                  src={META_BADGES[killmail.ship_meta_group_id].icon}
                />
              {/if}
            </div>
            <div class="flex items-center gap-2 px-2 py-1 text-lg whitespace-nowrap">
              <div
                class={[
                  'size-2 rounded-full',
                  killmail.character_id === character.id ? 'bg-red-500' : 'bg-green-500',
                ]}
              ></div>
              <p class="mr-auto font-bold">
                {formatIsk(killmail.value)}
              </p>
              {#if killmail.solo}
                <span class="text-neutral-500">solo</span>
              {:else}
                <span class="text-neutral-500">{killmail.killmail_involvements.length}</span>
              {/if}
            </div>
          </div>
        {/each}

        {#each placeholders as key (key)}
          <div
            data-slot
            class="w-32 shrink-0 border-r border-neutral-800 text-neutral-500"
            animate:flip={motion}
            in:fly={{ ...motion, x: -128 }}
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
        {/each}
      </div>

      {#if hasOverflow}
        <div
          class="absolute inset-y-0 right-0 w-32 border-l border-neutral-800 bg-neutral-900 text-neutral-500"
        >
          <div class="flex aspect-square items-center justify-center text-4xl font-extrabold">
            + {hidden}
          </div>
          <div class="flex items-center justify-center border-t border-neutral-800 py-2">
            <p class="text-center">{timeWindowLabel(hours)}</p>
          </div>
        </div>

        <div
          class="pointer-events-none absolute inset-y-0 right-32 w-16 bg-gradient-to-l from-neutral-900 to-transparent"
        ></div>
      {/if}
    </div>

    {#if summary}
      <div class="border-l border-neutral-800">
        <div class="relative h-full bg-neutral-900">
          {#if !hasOverflow}
            <div
              class="pointer-events-none absolute inset-y-0 right-full w-16 bg-gradient-to-l from-neutral-900 to-transparent"
            ></div>
          {/if}
          <div class="mx-8 grid h-full grid-cols-3 items-center gap-16 p-2">
            <div class="p-2">
              <h1
                class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500"
              >
                <span class="size-2 rounded-full bg-green-500"></span>
                ISK destroyed
              </h1>
              <p class="mb-4 text-right text-5xl font-extrabold text-white">
                {formatIsk(tweenedKills.current)}
              </p>
            </div>
            <div class="p-2">
              <h1
                class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500"
              >
                <span class="size-2 rounded-full bg-red-500"></span>
                ISK lost
              </h1>
              <p class="mb-4 text-right text-5xl font-extrabold text-white">
                {formatIsk(tweenedLosses.current)}
              </p>
            </div>
            <div class="p-2">
              <h1
                class="flex items-center gap-2 text-lg font-bold whitespace-nowrap text-neutral-500"
              >
                <span class="size-2 rounded-full bg-amber-500"></span>
                ISK efficiency
              </h1>
              <p class="mb-4 text-right text-5xl font-extrabold text-white">
                {efficiency.toFixed(0)}%
              </p>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
