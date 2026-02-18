<script lang="ts">
  import { onMount } from "svelte";
  import { slide } from "svelte/transition";
  import GlitchText from "./GlitchText.svelte";

  let {
    year = "",
    title = "",
    description = "",
    align = "left",
    dataRec = "85%",
  } = $props();

  let isVisible = $state(false);
  let isExpanded = $state(false);
  let cardRef: HTMLElement;

  onMount(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          isVisible = true;
          // Don't unobserve so it can trigger multiple times if needed,
          // or unobserve if you want it to stay visible.
          // observer.unobserve(cardRef);
        } else {
          // Optional: hide when scrolled out
          // isVisible = false;
        }
      },
      { threshold: 0.2 },
    );

    if (cardRef) observer.observe(cardRef);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={cardRef}
  class="relative mb-32 flex w-full {align === 'left'
    ? 'flex-row'
    : 'flex-row-reverse'} items-center opacity-0 transition-all duration-1000"
  class:opacity-100={isVisible}
  class:translate-y-0={isVisible}
  class:translate-y-10={!isVisible}
>
  <!-- Background Vertical Spine for this segment -->
  <div
    class="absolute left-1/2 -top-16 h-[calc(100%+8rem)] w-[1px] -translate-x-1/2 opacity-40"
  >
    <div
      class="h-full w-full border-l border-dotted border-[var(--text-secondary)]"
    ></div>
  </div>

  <!-- Main Content Area -->
  <div class="w-1/2 {align === 'left' ? 'pr-8 text-right' : 'pl-8 text-left'}">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="pixel-border z-10 w-full p-6 transition-all duration-500 hover:border-[var(--text-primary)] relative {description
        ? 'cursor-pointer'
        : ''}"
      onclick={() => {
        if (description) isExpanded = !isExpanded;
      }}
    >
      <!-- Connector Line to Spine -->
      <div
        class="absolute top-1/2 h-[1px] w-8 bg-[var(--text-secondary)] opacity-50 {align ===
        'left'
          ? '-right-8'
          : '-left-8'}"
      ></div>

      <div
        class="flex items-center justify-between border-[var(--text-secondary)] transition-all duration-300"
        class:mb-4={isExpanded}
        class:pb-2={isExpanded}
        class:border-b={isExpanded}
      >
        <h3
          class="glitch-hover pixel-text text-xl font-bold tracking-wider text-left"
        >
          {title}
        </h3>
        <span class="pixel-text text-xs opacity-50">[REC: {dataRec}]</span>
      </div>

      {#if isExpanded}
        <div
          class="serif-text text-lg text-[var(--text-secondary)] text-left max-h-96 overflow-y-auto custom-scrollbar pr-2"
          transition:slide={{ duration: 300 }}
          role="button"
          tabindex="0"
          onclick={(e) => e.stopPropagation()}
          onkeydown={(e) => e.stopPropagation()}
        >
          {@html description}
        </div>
      {/if}

      {#if description}
        <div class="mt-2 flex justify-end">
          <span
            class="pixel-text text-[10px] opacity-30 animate-pulse transition-transform duration-300"
            class:rotate-45={isExpanded}>[+]</span
          >
        </div>
      {/if}
    </div>
  </div>

  <!-- Central Spine Node -->
  <div
    class="absolute left-1/2 z-20 flex h-4 w-4 -translate-x-1/2 items-center justify-center"
  >
    <div
      class="h-2 w-2 border border-[var(--text-secondary)] bg-[var(--bg-color)] rotate-45 transition-all duration-700 delay-300"
      class:scale-150={isVisible}
      class:border-[var(--text-primary)]={isVisible}
    ></div>
    <!-- Glow Effect -->
    <div
      class="absolute h-4 w-4 rounded-full bg-[var(--text-primary)] opacity-0 blur-sm transition-opacity duration-1000 delay-500"
      class:opacity-20={isVisible}
    ></div>
  </div>

  <!-- Year Label (Opposite side) -->
  <div
    class="flex w-1/2 items-center {align === 'left'
      ? 'pl-8 text-left'
      : 'pr-8 text-right justify-end'} relative"
  >
    <!-- Small connector for year -->
    <div
      class="absolute top-1/2 h-[1px] w-4 bg-[var(--text-secondary)] opacity-30 {align ===
      'left'
        ? 'left-4'
        : 'right-4'}"
    ></div>
    <GlitchText text={year} delay={500} />
  </div>
</div>
