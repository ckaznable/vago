<script lang="ts">
  import EventCard from "./lib/EventCard.svelte";
  import DataGap from "./lib/DataGap.svelte";
  import fm from "front-matter";
  import { marked } from "marked";
  import pkg from "../package.json";

  interface EventData {
    year: string;
    title: string;
    description: string;
    dataRec: string;
  }

  type TimelineItem =
    | { type: "gap"; id: string; data?: undefined; align?: undefined }
    | { type: "event"; id: string; data: EventData; align: "left" | "right" };

  // Load all .md files from the /content directory
  const files = import.meta.glob("../content/*.md", {
    query: "?raw",
    eager: true,
  });

  const events: EventData[] = Object.values(files)
    .map((file: any) => {
      const { attributes, body } = fm<{
        year: string;
        title: string;
        dataRec: string;
      }>(file.default);
      return {
        year: attributes.year,
        title: attributes.title,
        dataRec: attributes.dataRec || "85%",
        description: marked.parse(body) as string,
      };
    })
    .sort((a, b) => {
      // Handle both year and full date
      const dateA = new Date(
        a.year.includes(":") ? a.year : `${a.year}-01-01`,
      ).getTime();
      const dateB = new Date(
        b.year.includes(":") ? b.year : `${b.year}-01-01`,
      ).getTime();
      return dateB - dateA;
    });

  let containerRef: HTMLElement;
  let scrollY = $state(0);

  // Create a fragmented timeline structure
  let timelineItems = $derived.by(() => {
    let items: TimelineItem[] = [];
    let eventCount = 0; // Track event count for strict L/R alternation

    // Add start gap
    items.push({ type: "gap", id: `gap-start` });

    for (let i = 0; i < events.length; i++) {
      // Add the event
      items.push({
        type: "event",
        data: events[i],
        id: `event-${i}`,
        align: eventCount % 2 === 0 ? "left" : "right",
      });
      eventCount++;

      // Add random gaps between items
      if (i < events.length - 1) {
        if (Math.random() > 0.4) {
          items.push({ type: "gap", id: `gap-${i}` });
        }
      }
    }
    // Add final gap
    items.push({ type: "gap", id: `gap-end` });
    return items;
  });
</script>

<svelte:window bind:scrollY />

<div class="crt-overlay"></div>

<main
  class="vago-pulse relative min-h-screen px-4 pb-64 pt-32 overflow-x-hidden"
>
  <!-- Header -->
  <header
    class="mb-32 flex flex-col items-center justify-center text-center relative z-20"
  >
    <div class="relative group cursor-help">
      <h1
        class="pixel-text mb-4 text-4xl tracking-[0.2em] md:text-7xl glitch-hover select-none"
      >
        WRD_ALLOC_LIMIT
      </h1>
      <div
        class="absolute -right-8 -top-4 pixel-text text-xs opacity-40 group-hover:text-[var(--accent-color)] transition-colors duration-300"
      >
        [VER {pkg.version}]
      </div>
    </div>

    <div
      class="h-[1px] w-48 bg-gradient-to-r from-transparent via-[var(--text-primary)] to-transparent mb-6 opacity-50"
    ></div>

    <div class="flex flex-col gap-1 items-center">
      <p
        class="pixel-text text-sm tracking-[0.5em] opacity-60 animate-pulse text-[var(--accent-color)]"
      >
        [ LOG: RECORDING... ]
      </p>
      <p class="pixel-text text-[10px] tracking-widest opacity-40">
        DATA_STREAM_ACTIVE // 紀錄進行中...
      </p>
    </div>
  </header>

  <div bind:this={containerRef} class="relative mx-auto max-w-3xl">
    <!-- Timeline Items -->
    {#each timelineItems as item (item.id)}
      {#if item.type === "gap"}
        <DataGap />
      {:else if item.type === "event"}
        <EventCard
          year={item.data.year}
          title={item.data.title}
          description={item.data.description}
          align={item.align}
          dataRec={item.data.dataRec}
        />
      {/if}
    {/each}
    <!-- Archive Init Marker -->
    <div class="relative flex flex-col items-center justify-center pb-32 pt-12">
      <!-- Connecting Spine -->
      <div
        class="absolute top-0 h-12 w-[1px] bg-[var(--text-secondary)] opacity-40"
      ></div>

      <div
        class="pixel-text text-[10px] tracking-[0.3em] text-[var(--accent-color)] animate-pulse"
      >
        [ ARCHIVE_INIT ]
      </div>
      <div class="mt-2 pixel-text text-[8px] tracking-widest opacity-50">
        SYSTEM_BOOT_SEQUENCE_COMPLETE
      </div>
    </div>
  </div>

  <footer
    class="border-t border-dashed border-[var(--text-secondary)] pt-12 text-center opacity-40"
  >
    <div
      class="pixel-text mb-4 text-[10px] tracking-widest text-[var(--text-secondary)]"
    >
      END_OF_LOG
    </div>
    <div class="pixel-text mb-4 text-[10px] tracking-widest">
      TERMINAL_LOG: CONNECTION_UNSTABLE // BUFFER_SYNC: 42%
    </div>
    <p class="pixel-text text-xs">
      &copy; 2026 ckaznable // [SESSION_ID: 0x{Math.floor(
        Math.random() * 0xffffff,
      )
        .toString(16)
        .toUpperCase()}]
    </p>
  </footer>
</main>

<style>
  :global(body) {
    background-color: #0d0f0e;
    color: #a8bba2;
  }
</style>
