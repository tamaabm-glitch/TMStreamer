<script lang="ts">
  let { small = false }: { small?: boolean } = $props();
</script>
<div class="flex items-center">
  <img src="/logo.svg" alt="TMStreamer" class={small ? "h-7 w-auto max-w-full object-contain" : "h-10 w-auto max-w-full object-contain"} />
</div>
