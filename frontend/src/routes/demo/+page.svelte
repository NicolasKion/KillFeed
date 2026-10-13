<script lang="ts">
  // The overlay with the simulated feed, for trying the options out.
  import { onDestroy, onMount, untrack } from 'svelte';

  import KillfeedOverlay from '#lib/components/killfeed-overlay.svelte';
  import { APP_NAME } from '#lib/killfeed.ts';
  import { createSimulatedKillfeed } from '#lib/simulated-killfeed.svelte.ts';

  let { data } = $props();

  const simulation = untrack(() =>
    createSimulatedKillfeed({ maxKillmails: data.count, ships: data.sim_ships }),
  );

  onMount(() => {
    simulation.seed(3);
    simulation.start();
  });
  onDestroy(() => simulation.stop());
</script>

<svelte:head>
  <title>Overlay demo - {APP_NAME}</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<KillfeedOverlay
  character={simulation.character}
  killmails={simulation.killmails}
  hours={data.hours}
  summary={data.summary}
  showMeta={data.meta}
  killsValue={simulation.killsValue}
  lossesValue={simulation.lossesValue}
  killmailsCount={simulation.killmailsCount}
/>

<style>
  :global(html),
  :global(body),
  :global(#app) {
    height: 100%;
    margin: 0;
    overflow: hidden;
  }
</style>
