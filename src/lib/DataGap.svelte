<script lang="ts">
  import { onMount } from "svelte";

  let chars = "01";
  let binaryStream = $state("");
  let isVisible = $state(false);
  let gapRef: HTMLElement;

  onMount(() => {
    // Generate random binary stream
    binaryStream = Array(24)
      .fill(0)
      .map(() => chars[Math.floor(Math.random() * chars.length)])
      .join("");

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          isVisible = true;
          observer.disconnect();
        }
      },
      { threshold: 0.1 },
    );
    if (gapRef) observer.observe(gapRef);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={gapRef}
  class="relative flex w-full flex-col items-center justify-center py-8 z-10 transition-opacity duration-1000"
  class:opacity-100={isVisible}
  class:opacity-0={!isVisible}
>
  <!-- Background Vertical Spine for this segment -->
  <div
    class="absolute left-1/2 -top-16 h-[calc(100%+8rem)] w-[1px] -translate-x-1/2 opacity-40"
  >
    <div
      class="h-full w-full border-l border-dotted border-[var(--text-secondary)]"
    ></div>
  </div>

  <!-- Glitch Node on Spine -->
  <div
    class="relative flex flex-col items-center bg-[var(--bg-color)] p-2 border border-dashed border-[var(--text-secondary)] z-20"
  >
    <span
      class="pixel-text text-[8px] text-[var(--accent-color)] tracking-widest animate-pulse mb-1"
    >
      ⚠ LOST
    </span>
    <div
      class="pixel-text w-32 text-center break-all text-[8px] leading-none opacity-40 text-[var(--text-secondary)]"
    >
      {binaryStream}
    </div>
  </div>
</div>
