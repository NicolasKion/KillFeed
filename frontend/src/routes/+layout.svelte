<script lang="ts">
  import '../app.css';
  import { page } from '$app/state';

  let { children } = $props();

  // The current URL without its query and trailing slash.
  const currentUrl = $derived((page.url.origin + page.url.pathname).replace(/\/+$/, ''));
</script>

<svelte:head>
  <meta property="og:image" content="{page.url.origin}/og.png" />
  <meta property="og:url" content={currentUrl} />
</svelte:head>

{@render children()}
