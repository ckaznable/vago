<script lang="ts">
  import { onMount } from 'svelte';

  let { text = '', delay = 0 } = $props();
  let displayedText = $state('');
  let isVisible = $state(false);
  let container: HTMLElement;

  const chars = '0123456789ABCDEF!@#$%^&*()_+';

  function decode() {
    let iteration = 0;
    const interval = setInterval(() => {
      displayedText = text
        .split('')
        .map((char, index) => {
          if (index < iteration) {
            return text[index];
          }
          return chars[Math.floor(Math.random() * chars.length)];
        })
        .join('');

      if (iteration >= text.length) {
        clearInterval(interval);
      }

      iteration += 1 / 3;
    }, 30);
  }

  onMount(() => {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting && !isVisible) {
        isVisible = true;
        setTimeout(decode, delay);
      }
    });

    if (container) observer.observe(container);
    return () => observer.disconnect();
  });
</script>

<span bind:this={container} class="pixel-text inline-block min-w-[4ch]">
  {displayedText || ' '}
</span>
