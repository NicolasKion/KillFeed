<script lang="ts">
  // The overlay OBS shows. It reloads its props every 10 seconds, which
  // also keeps the character watched by the firehose listener.
  import { onMount } from 'svelte';
  import { invalidateAll } from '$app/navigation';

  import KillfeedOverlay from '#lib/components/killfeed-overlay.svelte';
  import { APP_NAME } from '#lib/killfeed.ts';

  let { data } = $props();

  onMount(() => {
    const interval = setInterval(() => {
      invalidateAll();
    }, 10_000);
    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>{data.character.name} - {APP_NAME}</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<KillfeedOverlay
  character={data.character}
  killmails={data.killmails}
  hours={data.hours}
  summary={data.summary}
  showMeta={data.meta}
  killsValue={data.kills_value}
  lossesValue={data.losses_value}
  killmailsCount={data.killmails_count}
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
