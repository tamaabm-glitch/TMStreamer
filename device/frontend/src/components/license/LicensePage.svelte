<script lang="ts">
  import { license } from "@/stores/license.svelte";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import KeyRound from "@lucide/svelte/icons/key-round";
  import Copy from "@lucide/svelte/icons/copy";
  import Upload from "@lucide/svelte/icons/upload";

  let { manage = false }: { manage?: boolean } = $props();

  let selected = $state<File | null>(null);
  let notice = $state<string | null>(null);

  async function copyMachineId() {
    if (!license.machineId) return;
    try {
      await navigator.clipboard.writeText(license.machineId);
      notice = "Machine ID berhasil disalin.";
    } catch {
      notice = "Salin Machine ID secara manual.";
    }
  }

  async function activate() {
    if (!selected) return;
    try {
      await license.activate(selected);
      selected = null;
      notice = "Lisensi TMStreamer berhasil diaktifkan.";
    } catch {
      // The store exposes the verification error.
    }
  }
</script>

<div class="mx-auto flex w-full max-w-xl flex-col gap-5 py-8">
  <section class="rounded-2xl border border-line bg-surface p-6 shadow-sm">
    <div class="mb-5 flex items-center gap-3">
      <div
        class="grid h-12 w-12 place-items-center rounded-xl bg-primary/10 text-primary"
      >
        <ShieldCheck class="h-6 w-6" aria-hidden="true" />
      </div>
      <div>
        <p
          class="text-[11px] font-semibold tracking-[0.18em] text-primary uppercase"
        >
          TMStreamer License
        </p>
        {#if license.licensed && !manage}
          <h1 class="text-xl font-semibold text-fg">
            Fitur tidak termasuk paket
          </h1>
        {:else if license.licensed}
          <h1 class="text-xl font-semibold text-fg">
            Lisensi TMStreamer aktif
          </h1>
        {:else}
          <h1 class="text-xl font-semibold text-fg">Aktivasi diperlukan</h1>
        {/if}
      </div>
    </div>

    {#if license.licensed && !manage}
      <p class="text-sm leading-6 text-muted">
        Lisensi {license.status?.tier ?? ""} sudah aktif, tetapi paket ini belum mencakup
        fitur yang sedang dibuka. Fitur aktif:
        {license.status?.features.join(", ") || "tidak ada"}. Unggah lisensi
        dengan hak fitur yang sesuai untuk memperbarui paket perangkat.
      </p>
    {:else if license.licensed}
      <p class="text-sm leading-6 text-muted">
        Lisensi {license.status?.tier ?? ""} aktif. Fitur tersedia:
        {license.status?.features.join(", ") || "tidak ada"}. Kamu dapat
        mengunggah file lisensi baru untuk memperbarui hak fitur perangkat.
      </p>
    {:else}
      <p class="text-sm leading-6 text-muted">
        Masukkan file lisensi resmi yang diterbitkan untuk perangkat ini. Fitur
        yang tercantum dalam lisensi akan terbuka setelah tanda tangan digital
        berhasil diverifikasi.
      </p>
    {/if}

    <div class="mt-6 rounded-xl border border-line bg-surface-2 p-4">
      <div class="mb-2 flex items-center gap-2 text-sm font-medium text-fg">
        <KeyRound class="h-4 w-4 text-primary" aria-hidden="true" />
        Machine ID perangkat
      </div>
      {#if license.machineId}
        <div class="flex flex-wrap items-center gap-2">
          <code
            class="min-w-0 flex-1 rounded-md bg-surface px-3 py-2 text-sm break-all text-fg"
          >
            {license.machineId}
          </code>
          <button
            type="button"
            onclick={copyMachineId}
            class="inline-flex items-center gap-2 rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface"
          >
            <Copy class="h-3.5 w-3.5" aria-hidden="true" />
            Salin
          </button>
        </div>
      {:else}
        <p class="text-sm text-muted">
          {license.status?.message ??
            license.error ??
            "Machine ID belum tersedia."}
        </p>
      {/if}
    </div>

    <div class="mt-6 flex flex-col gap-3">
      <label class="text-sm font-medium text-fg" for="tm-license-file">
        File lisensi (.json)
      </label>
      <input
        id="tm-license-file"
        type="file"
        accept=".json,application/json"
        onchange={(event) =>
          (selected = event.currentTarget.files?.[0] ?? null)}
        class="block w-full rounded-lg border border-line bg-surface px-3 py-3 text-sm text-muted file:mr-3 file:rounded-md file:border-0 file:bg-surface-2 file:px-3 file:py-2 file:text-xs file:font-semibold file:text-fg"
      />
      <button
        type="button"
        onclick={activate}
        disabled={!selected || !license.machineId || license.activating}
        class="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-primary px-4 py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-40"
      >
        <Upload class="h-4 w-4" aria-hidden="true" />
        {license.activating
          ? "Memverifikasi lisensi…"
          : license.licensed
            ? "Perbarui lisensi TMStreamer"
            : "Aktifkan TMStreamer"}
      </button>
    </div>

    {#if license.status?.message}
      <p
        role="alert"
        class="mt-4 rounded-lg border border-warn/30 bg-warn/5 px-3 py-2 text-sm text-warn"
      >
        {license.status.message}
      </p>
    {/if}
    {#if license.error}
      <p
        role="alert"
        class="mt-4 rounded-lg border border-down/30 bg-down/5 px-3 py-2 text-sm text-down"
      >
        {license.error}
      </p>
    {/if}
    {#if notice}
      <p role="status" class="mt-3 text-sm text-muted">{notice}</p>
    {/if}
  </section>

  <section class="rounded-xl border border-line bg-surface p-5">
    <h2 class="mb-2 text-sm font-semibold text-fg">Pengaturan perangkat</h2>
    <p class="mb-4 text-xs leading-5 text-muted">
      Koneksi jaringan dan pengaturan HDMI tetap dapat diakses selama proses
      aktivasi.
    </p>
    <div class="flex flex-wrap gap-2">
      {#if license.canRecord}
        <a
          href="/recording"
          class="rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface-2"
          >Recording</a
        >
      {/if}
      <a
        href="/input"
        class="rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface-2"
        >DJI Input</a
      >
      <a
        href="/network"
        class="rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface-2"
      >
        Network setup
      </a>
      <a
        href="/display"
        class="rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface-2"
      >
        HDMI / Display
      </a>
      <a
        href="/system"
        class="rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface-2"
      >
        System & Updates
      </a>
      <a
        href="/login"
        class="rounded-md border border-line px-3 py-2 text-xs font-medium text-fg hover:bg-surface-2"
        >Log in</a
      >
    </div>
  </section>
</div>
