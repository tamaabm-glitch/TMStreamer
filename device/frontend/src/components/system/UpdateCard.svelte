<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "@/api/client";
  import type { UpdateStatus } from "@/types/system";
  import type { Settings } from "@/types/settings";
  import Card from "@/components/ui/Card.svelte";
  import Field from "@/components/ui/Field.svelte";

  // Keep the existing prop contract; automatic checks are intentionally disabled
  // until TMStreamer-owned signed update infrastructure is available.
  let { advanced: _advanced }: { advanced: Settings["advanced"] } = $props();

  let status = $state<UpdateStatus | null>(null);
  let unavailable = $state(false);

  onMount(async () => {
    try {
      status = await api.update();
    } catch {
      unavailable = true;
    }
  });
</script>

<Card title="Updates">
  {#if status}
    <Field label="Installed version">
      <span class="text-[13px] text-muted tabular-nums">{status.current}</span>
    </Field>
    {#if status.failure}
      <div class="border-t border-line px-4 py-3">
        <p class="text-[13px] text-down">
          {#if status.failure.reverted}
            Version {status.failure.version} did not start. The device returned
            to {status.current}.
          {:else}
            Installing version {status.failure.version} failed.
          {/if}
        </p>
        {#if status.failure.log}
          <pre
            class="mt-2 max-h-48 overflow-auto rounded-lg bg-surface-2 p-3 font-mono text-[11px] whitespace-pre-wrap text-muted">{status
              .failure.log}</pre>
        {/if}
      </div>
    {/if}
  {:else if unavailable}
    <p class="px-4 py-3 text-[13px] text-muted">
      Update status is currently unavailable.
    </p>
  {:else}
    <p class="px-4 py-3 text-[13px] text-muted">Reading installed version…</p>
  {/if}

  <div class="border-t border-line px-4 py-3">
    <p class="text-[13px] text-muted">
      TMStreamer updates are not enabled yet. Checking, downloading, installing,
      uploading packages, and rollback will remain unavailable until the
      TMStreamer-owned signing key and update service are configured and tested.
    </p>
  </div>
</Card>
