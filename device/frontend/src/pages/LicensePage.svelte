<script lang="ts">
  import { onMount } from "svelte";
  import { api, message } from "@/api/client";
  import type { LicenseStatus } from "@/types/license";
  import { btn } from "@/components/ui/buttons";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import Upload from "@lucide/svelte/icons/upload";
  import LockKeyhole from "@lucide/svelte/icons/lock-keyhole";

  let status = $state<LicenseStatus | null>(null);
  let selected = $state<File | null>(null);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);

  const licensed = $derived(
    status?.product === "TMSTREAMER" &&
    status?.license_system === "TM-LIC" &&
    status?.activated === true &&
    status?.status === "LICENSED"
  );

  async function refresh() {
    loading = true;
    try {
      status = await api.licenseStatus();
    } catch (e) {
      status = null;
      error = message(e);
    } finally {
      loading = false;
    }
  }

  function chooseFile(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    selected = target.files?.[0] ?? null;
    error = null;
    notice = null;
  }

  async function activate() {
    if (!selected || busy) return;
    busy = true;
    error = null;
    notice = null;
    try {
      status = await api.activateLicense(selected);
      selected = null;
      notice = "Lisensi TM-LIC berhasil diverifikasi dan diaktifkan.";
      const input = document.getElementById("tm-license-file") as HTMLInputElement | null;
      if (input) input.value = "";
    } catch (e) {
      error = message(e);
    } finally {
      busy = false;
      await refresh();
    }
  }

  onMount(refresh);
</script>

<div class="mx-auto flex w-full max-w-2xl flex-col gap-5">
  <section class="overflow-hidden rounded-2xl border border-line bg-surface shadow-sm">
    <div class="flex items-center gap-3 border-b border-line px-5 py-4">
      <span class="grid h-11 w-11 place-items-center rounded-xl bg-primary/10 text-primary">
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </span>
      <div class="min-w-0">
        <h2 class="text-base font-semibold text-fg">TM-LIC License</h2>
        <p class="text-xs text-muted">Sistem lisensi resmi TMStreamer</p>
      </div>
      <span class="ml-auto rounded-full px-3 py-1 text-xs font-semibold {licensed ? 'bg-up/10 text-up' : 'bg-warn/10 text-warn'}">
        {#if loading}Memeriksa…{:else}{status?.status ?? "UNVERIFIED"}{/if}
      </span>
    </div>

    <div class="flex flex-col gap-4 p-5">
      {#if licensed}
        <div class="flex items-start gap-3 rounded-xl bg-up/5 p-4">
          <ShieldCheck class="mt-0.5 h-5 w-5 shrink-0 text-up" aria-hidden="true" />
          <div>
            <p class="text-sm font-semibold text-fg">Lisensi aktif</p>
            <p class="mt-1 text-xs text-muted">Perangkat ini telah lolos verifikasi TM-LIC.</p>
          </div>
        </div>
      {:else}
        <div class="flex items-start gap-3 rounded-xl bg-warn/5 p-4">
          <LockKeyhole class="mt-0.5 h-5 w-5 shrink-0 text-warn" aria-hidden="true" />
          <div>
            <p class="text-sm font-semibold text-fg">Fitur berlisensi terkunci</p>
            <p class="mt-1 text-xs leading-5 text-muted">
              Streaming dan DVR hanya dapat digunakan setelah file lisensi TM-LIC yang valid untuk perangkat ini diverifikasi.
              Pengaturan perangkat, receiver, jaringan, dan HDMI tetap dapat diakses.
            </p>
          </div>
        </div>
      {/if}

      {#if status?.tier}
        <dl class="grid grid-cols-1 gap-3 rounded-xl bg-surface-2 p-4 sm:grid-cols-2">
          <div><dt class="text-[11px] text-muted">Tier</dt><dd class="mt-1 text-sm font-medium text-fg">{status.tier}</dd></div>
          <div><dt class="text-[11px] text-muted">License ID</dt><dd class="mt-1 break-all font-mono text-xs text-fg">{status.license_id ?? "—"}</dd></div>
          <div><dt class="text-[11px] text-muted">Berlaku sampai</dt><dd class="mt-1 text-sm text-fg">{status.expires_at ?? "Tidak kedaluwarsa"}</dd></div>
          <div><dt class="text-[11px] text-muted">Key ID</dt><dd class="mt-1 font-mono text-xs text-fg">{status.key_id ?? "—"}</dd></div>
          <div class="sm:col-span-2"><dt class="text-[11px] text-muted">Fitur aktif</dt><dd class="mt-1 text-xs leading-5 text-fg">{status.features.join(", ") || "Tidak ada"}</dd></div>
        </dl>
      {/if}

      {#if status?.message}
        <p class="text-xs text-muted">{status.message}</p>
      {/if}
      {#if error}
        <p class="rounded-lg bg-down/10 p-3 text-xs text-down">{error}</p>
      {/if}
      {#if notice}
        <p class="rounded-lg bg-up/10 p-3 text-xs text-up">{notice}</p>
      {/if}

      <div class="flex flex-col gap-2 border-t border-line pt-4">
        <label for="tm-license-file" class="text-sm font-medium text-fg">File lisensi TM-LIC (.json)</label>
        <input
          id="tm-license-file"
          type="file"
          accept=".json,application/json"
          onchange={chooseFile}
          class="block w-full rounded-lg border border-line bg-surface-2 p-2 text-xs text-fg file:mr-3 file:rounded-md file:border-0 file:bg-primary file:px-3 file:py-2 file:text-xs file:font-semibold file:text-white"
        />
        <p class="text-[11px] leading-5 text-muted">File harus ditandatangani penerbit TM-LIC dan terikat pada machine ID Radxa ini. File yang tidak valid tidak akan mengganti lisensi yang sudah terpasang.</p>
        <div class="flex flex-wrap gap-2">
          <button type="button" class={btn.primary} disabled={!selected || busy} onclick={activate}>
            <Upload class="mr-2 inline h-4 w-4" aria-hidden="true" />
            {busy ? "Memverifikasi…" : "Verifikasi & Aktivasi"}
          </button>
          <button type="button" class={btn.plain} disabled={loading || busy} onclick={refresh}>Periksa ulang</button>
        </div>
      </div>
    </div>
  </section>
</div>
