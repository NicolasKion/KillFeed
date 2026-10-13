<script lang="ts">
  // The setup page: the overlay running on simulated killmails, then the
  // three steps that build a streamer's OBS browser source URL, with a
  // live preview once a character id is found.
  import { onDestroy, onMount, untrack } from 'svelte';

  import KillfeedOverlay from '#lib/components/killfeed-overlay.svelte';
  import { APP_NAME, eveImageUrl } from '#lib/killfeed.ts';
  import { chatCommandMessage, clamp, overlayQuery, overlayUrl, safeId } from '#lib/setup.ts';
  import { createSimulatedKillfeed } from '#lib/simulated-killfeed.svelte.ts';
  import mutamarketLogo from '#lib/assets/support/mutamarket.svg';
  import patreonLogo from '#lib/assets/support/patreon.png';
  import wormholeLogo from '#lib/assets/support/wormhole-dark.png';

  type CharacterPreview = {
    id: number;
    name: string;
    portrait: string;
  };

  let { data } = $props();

  const maxVisibleKillmails = $derived(Math.max(1, data.max_visible_killmails));
  const maxKillmailAgeHours = $derived(Math.max(1, data.max_killmail_age_hours));
  const discordInviteUrl = $derived(data.discord_invite_url?.trim() ?? '');

  let id = $state('');
  // The options start at the maximums; the inputs change them from here.
  let count = $state(untrack(() => Math.max(1, data.max_visible_killmails)));
  let hours = $state(untrack(() => Math.max(1, data.max_killmail_age_hours)));
  let summary = $state(true);
  let meta = $state(true);

  let character = $state<CharacterPreview | null>(null);
  let isLoadingCharacter = $state(false);
  let characterError = $state('');
  let copied = $state<'url' | 'isk' | 'chat' | null>(null);
  let origin = $state('');
  let previewFrameSrc = $state('');

  let debounceTimer: ReturnType<typeof setTimeout> | undefined;
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;
  let frameTimer: ReturnType<typeof setTimeout> | undefined;

  const simulation = createSimulatedKillfeed({ ships: untrack(() => data.sim_ships) });

  const settings = $derived({ count, hours, summary, meta });
  const previewPath = $derived(overlayUrl(origin, id, settings));
  const demoPath = $derived(`/demo?${overlayQuery(settings)}`);
  const canPreview = $derived(character !== null && previewPath.length > 0);
  const chatMessage = $derived(chatCommandMessage(origin));

  // The iframe follows the options with a debounce, so typing in the
  // count or hours inputs does not reload the overlay per keystroke.
  let firstFrame = true;
  $effect(() => {
    const value = previewPath;
    if (firstFrame) {
      firstFrame = false;
      return;
    }
    clearTimeout(frameTimer);
    frameTimer = setTimeout(() => {
      previewFrameSrc = value;
    }, 350);
  });

  // Pause the simulation while the real feed is on screen.
  let wasLive = false;
  $effect(() => {
    const live = canPreview;
    if (live === wasLive) {
      return;
    }
    wasLive = live;
    if (live) {
      simulation.stop();
    } else {
      simulation.start();
    }
  });

  function lookUp(value: string): void {
    character = null;
    characterError = '';
    clearTimeout(debounceTimer);

    if (value.length === 0) {
      isLoadingCharacter = false;
      return;
    }

    isLoadingCharacter = true;
    debounceTimer = setTimeout(async () => {
      try {
        const response = await fetch(`/characters/${value}/preview`, {
          headers: { Accept: 'application/json' },
        });
        if (!response.ok) {
          throw new Error('Character not found');
        }
        character = (await response.json()) as CharacterPreview;
      } catch {
        characterError =
          'Character not found. Check the character ID from zKillboard and try again.';
      } finally {
        isLoadingCharacter = false;
      }
    }, 350);
  }

  function typeId(event: Event & { currentTarget: HTMLInputElement }): void {
    const clean = safeId(event.currentTarget.value);
    event.currentTarget.value = clean;
    if (clean !== id) {
      id = clean;
      lookUp(clean);
    }
  }

  function typeCount(event: Event & { currentTarget: HTMLInputElement }): void {
    count = clamp(Number.parseFloat(event.currentTarget.value), maxVisibleKillmails);
    event.currentTarget.value = String(count);
  }

  function typeHours(event: Event & { currentTarget: HTMLInputElement }): void {
    hours = clamp(Number.parseFloat(event.currentTarget.value), maxKillmailAgeHours);
    event.currentTarget.value = String(hours);
  }

  async function copyText(value: string): Promise<boolean> {
    if (typeof navigator === 'undefined' || !navigator.clipboard || value.length === 0) {
      return false;
    }
    try {
      await navigator.clipboard.writeText(value);
      return true;
    } catch {
      return false;
    }
  }

  async function copyWithFeedback(key: 'url' | 'isk' | 'chat', value: string): Promise<void> {
    if (!(await copyText(value))) {
      return;
    }
    copied = key;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copied = null;
    }, 1600);
  }

  onMount(() => {
    origin = window.location.origin;
    simulation.seed(2);
    simulation.start();
  });

  onDestroy(() => {
    simulation.stop();
    clearTimeout(copiedTimer);
    clearTimeout(frameTimer);
    clearTimeout(debounceTimer);
  });
</script>

<svelte:head>
  <title>EVE Online Stream Overlay - {APP_NAME}</title>
</svelte:head>

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
        <p class="font-display text-lg font-bold tracking-[0.25em] uppercase">
          Kill<span class="text-signal">Feed</span>
        </p>
        <p class="hidden text-sm text-drift sm:block">EVE Online kill overlay for streamers</p>
      </div>
      {#if discordInviteUrl}
        <a
          href={discordInviteUrl}
          class="font-display text-xs font-semibold tracking-[0.15em] text-drift uppercase transition-colors hover:text-starlight"
          rel="noopener noreferrer"
          target="_blank"
        >
          Discord
        </a>
      {/if}
    </header>

    <!-- Hero: the overlay itself, live -->
    <section class="py-10 sm:py-14">
      <h1 class="max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
        Put your kills on stream.
      </h1>
      <p class="mt-4 max-w-2xl text-base leading-relaxed text-drift">
        KillFeed turns a character's live killmails into an OBS browser source. The feed below is a
        simulation — enter your character ID and it switches to the real thing.
      </p>

      <!-- Target-lock frame -->
      <div class="relative mt-10">
        <span
          aria-hidden="true"
          class="absolute -top-px -left-px size-4 border-t-2 border-l-2 border-signal"
        ></span>
        <span
          aria-hidden="true"
          class="absolute -top-px -right-px size-4 border-t-2 border-r-2 border-signal"
        ></span>
        <span
          aria-hidden="true"
          class="absolute -bottom-px -left-px size-4 border-b-2 border-l-2 border-signal"
        ></span>
        <span
          aria-hidden="true"
          class="absolute -right-px -bottom-px size-4 border-r-2 border-b-2 border-signal"
        ></span>

        <div class="bg-black p-3">
          {#if canPreview}
            {#key previewFrameSrc}
              <iframe
                src={previewFrameSrc}
                class="min-h-[166px] w-full"
                loading="eager"
                scrolling="no"
                title="KillFeed preview"
              ></iframe>
            {/key}
          {:else}
            <KillfeedOverlay
              character={simulation.character}
              {hours}
              killmails={simulation.killmails}
              killmailsCount={simulation.killmailsCount}
              killsValue={simulation.killsValue}
              lossesValue={simulation.lossesValue}
              showMeta={meta}
              {summary}
            />
          {/if}
        </div>
      </div>

      <!-- Console status line -->
      <div class="mt-3 flex flex-wrap items-center justify-between gap-3 font-mono text-xs">
        {#if canPreview}
          <p class="flex items-center gap-2 text-drift">
            <span class="size-2 rounded-full bg-confirm motion-safe:animate-pulse"></span>
            <span class="font-semibold text-confirm">LIVE</span>
            tracking {character?.name} · {character?.id}
          </p>
        {:else}
          <p class="flex items-center gap-2 text-drift">
            <span class="size-2 rounded-full bg-signal motion-safe:animate-pulse"></span>
            <span class="font-semibold text-signal">SIM</span>
            synthetic killmails · a new kill lands every few seconds
          </p>
        {/if}
        <a
          href={demoPath}
          class="text-drift underline decoration-plate underline-offset-4 transition-colors hover:text-starlight"
        >
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
          value={id}
          oninput={typeId}
          class="mt-2 w-full border-plate bg-hull px-3 py-2 font-mono text-sm text-starlight focus:border-drift focus:ring-0"
          inputmode="numeric"
          name="character-id"
          placeholder="2112625428"
          type="text"
        />
        <p class="mt-3 text-sm leading-relaxed text-drift">
          Find it in any zKillboard URL: <span class="font-mono text-xs"
            >zkillboard.com/character/&lt;id&gt;</span
          >
        </p>

        <div class="mt-6 border border-plate bg-hull p-4">
          <p class="font-mono text-xs text-drift">character status</p>
          {#if isLoadingCharacter}
            <p class="mt-2 text-sm text-drift">Looking up character...</p>
          {:else if characterError}
            <p class="mt-2 text-sm text-signal">
              {characterError}
            </p>
          {:else if character}
            <div class="mt-3 flex items-center gap-3">
              <img
                alt={character.name}
                src={character.portrait}
                class="size-12 border border-plate"
              />
              <div>
                <p class="text-sm font-semibold">{character.name}</p>
                <p class="font-mono text-xs text-drift">{character.id}</p>
              </div>
            </div>
          {:else}
            <p class="mt-2 text-sm text-drift">Enter a character ID to begin.</p>
          {/if}
        </div>
      </div>

      <!-- 02 Configure -->
      <div class="border-b border-plate py-8 lg:border-r lg:border-b-0 lg:px-8">
        <p class="font-display text-xs font-semibold tracking-[0.2em] text-drift uppercase">
          <span class="text-signal">02</span>
          Configure
        </p>

        <label class="mt-6 block text-sm font-medium" for="visible-killmails"
          >Visible killmails</label
        >
        <input
          id="visible-killmails"
          value={count}
          oninput={typeCount}
          class="mt-2 w-full border-plate bg-hull px-3 py-2 font-mono text-sm text-starlight focus:border-drift focus:ring-0"
          max={maxVisibleKillmails}
          min="1"
          name="visible-killmails"
          type="number"
        />
        <p class="mt-2 text-sm leading-relaxed text-drift">
          Caps the cards on screen; extra kills collapse into a +N tile. Maximum {maxVisibleKillmails}.
        </p>

        <label class="mt-6 block text-sm font-medium" for="killmail-age">Killmail age (hours)</label
        >
        <input
          id="killmail-age"
          value={hours}
          oninput={typeHours}
          class="mt-2 w-full border-plate bg-hull px-3 py-2 font-mono text-sm text-starlight focus:border-drift focus:ring-0"
          max={maxKillmailAgeHours}
          min="1"
          name="killmail-age"
          type="number"
        />
        <p class="mt-2 text-sm leading-relaxed text-drift">
          Only kills newer than this show up, in cards and totals. Maximum {maxKillmailAgeHours} hours.
        </p>

        <label class="mt-6 flex items-center gap-3">
          <input
            bind:checked={summary}
            class="border-plate bg-hull text-signal focus:ring-0 focus:ring-offset-0"
            type="checkbox"
          />
          <span class="text-sm font-medium">Include summary panel</span>
        </label>
        <p class="mt-2 text-sm leading-relaxed text-drift">
          Adds ISK destroyed, ISK lost, and efficiency. Try it on the feed above.
        </p>

        <label class="mt-6 flex items-center gap-3">
          <input
            bind:checked={meta}
            class="border-plate bg-hull text-signal focus:ring-0 focus:ring-offset-0"
            type="checkbox"
          />
          <span class="text-sm font-medium">Show meta group icons</span>
        </label>
        <p class="mt-2 text-sm leading-relaxed text-drift">
          Marks Tech II, Tech III, faction, storyline, and officer hulls with their in-game corner
          icon.
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
          class={[
            'mt-2 flex min-h-10 items-center overflow-x-auto border border-plate bg-hull px-3 py-2 font-mono text-xs whitespace-nowrap',
            previewPath ? 'text-starlight' : 'text-drift',
          ]}
        >
          {previewPath || 'waiting for a valid character ID...'}
        </code>

        <button
          disabled={!canPreview}
          class="mt-4 w-full cursor-pointer border border-plate bg-hull px-4 py-2.5 font-display text-xs font-semibold tracking-[0.15em] uppercase transition-colors hover:border-drift disabled:cursor-not-allowed disabled:opacity-40"
          type="button"
          onclick={() => copyWithFeedback('url', previewPath)}
        >
          {copied === 'url' ? 'Copied' : 'Copy URL'}
        </button>

        <p class="mt-4 text-sm leading-relaxed text-drift">
          Add it as a browser source in OBS and stretch it across the bottom of your scene. The feed
          updates itself every 10 seconds.
        </p>
      </div>
    </section>

    <!-- Support -->
    <section class="my-10 border border-plate bg-hull px-6 py-8 sm:px-10 sm:py-10">
      <p class="font-display text-xs font-semibold tracking-[0.2em] text-confirm uppercase">
        Support the project
      </p>
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
          <span
            class="grid size-10 shrink-0 place-items-center overflow-hidden border border-plate bg-hull"
          >
            <img alt="" class="size-10" src={patreonLogo} />
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
            <span class="mt-1 block text-sm leading-relaxed text-drift"
              >Monthly support for hosting and development.</span
            >
          </span>
        </a>
        <button
          class="group flex cursor-pointer items-start gap-4 border border-plate bg-void p-4 text-left transition-colors hover:border-confirm/60 hover:bg-hull"
          type="button"
          onclick={() => copyWithFeedback('isk', 'Nicolas Kion')}
        >
          <span
            class="grid size-10 shrink-0 place-items-center overflow-hidden border border-plate bg-hull"
          >
            <img
              alt="Nicolas Kion"
              class="size-10"
              src={eveImageUrl('characters', 95569887, 'portrait', 64)}
            />
          </span>
          <span class="min-w-0">
            <span class={['block font-medium', copied === 'isk' && 'text-confirm']}>
              {copied === 'isk' ? 'Copied to clipboard' : 'In-game ISK'}
            </span>
            <span class="mt-1 block text-sm leading-relaxed text-drift"
              >Send ISK to Nicolas Kion. Click to copy the name.</span
            >
          </span>
        </button>
        <a
          class="group flex items-start gap-4 border border-plate bg-void p-4 transition-colors hover:border-confirm/60 hover:bg-hull"
          href="https://mutamarket.com"
          rel="noopener noreferrer"
          target="_blank"
        >
          <span class="grid size-10 shrink-0 place-items-center border border-plate bg-hull">
            <img alt="" class="size-6" src={mutamarketLogo} />
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
            <span class="mt-1 block text-sm leading-relaxed text-drift"
              >EVE market tools from the same developer.</span
            >
          </span>
        </a>
        <a
          class="group flex items-start gap-4 border border-plate bg-void p-4 transition-colors hover:border-confirm/60 hover:bg-hull"
          href="https://wormhole.systems"
          rel="noopener noreferrer"
          target="_blank"
        >
          <span class="grid size-10 shrink-0 place-items-center border border-plate bg-hull">
            <img alt="" class="size-6" src={wormholeLogo} />
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
            <span class="mt-1 block text-sm leading-relaxed text-drift"
              >Wormhole tooling from the same developer.</span
            >
          </span>
        </a>
      </div>

      <div class="mt-8 border border-plate bg-void p-5">
        <p class="font-display text-xs font-semibold tracking-[0.2em] text-drift uppercase">
          Spread it in chat
        </p>
        <p class="mt-2 max-w-2xl text-sm leading-relaxed text-drift">
          Viewers will ask about the overlay mid-fight. A <span class="font-mono text-starlight"
            >!killfeed</span
          > chat command answers for you and sends them here to set up their own.
        </p>
        <div class="mt-3 flex flex-wrap items-stretch gap-3">
          <code
            class="flex min-h-10 grow items-center overflow-x-auto border border-plate bg-hull px-3 py-2 font-mono text-xs whitespace-nowrap text-starlight"
          >
            {chatMessage}
          </code>
          <button
            class={[
              'cursor-pointer border border-plate bg-hull px-4 py-2 font-display text-xs font-semibold tracking-[0.15em] uppercase transition-colors hover:border-drift',
              copied === 'chat' && 'text-confirm',
            ]}
            type="button"
            onclick={() => copyWithFeedback('chat', chatMessage)}
          >
            {copied === 'chat' ? 'Copied' : 'Copy message'}
          </button>
        </div>
      </div>
    </section>

    <footer class="border-t border-plate py-6">
      <p class="font-mono text-xs text-drift">
        KillFeed is a community project. EVE Online and all related assets are the property of CCP
        hf.
      </p>
    </footer>
  </div>
</main>
