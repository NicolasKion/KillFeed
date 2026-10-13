<script lang="ts">
  import '../app.css';
  import { page } from '$app/state';

  let { children } = $props();

  // The current URL without its query, as Laravel's `url()->current()`.
  const currentUrl = $derived(page.url.origin + page.url.pathname);
</script>

<svelte:head>
  <meta property="og:image" content="{page.url.origin}/og.png" />
  <meta property="og:url" content={currentUrl} />
</svelte:head>

{@render children()}
