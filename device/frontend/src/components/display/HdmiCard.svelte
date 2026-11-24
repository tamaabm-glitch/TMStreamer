<script lang="ts">
  import { onMount } from "svelte";
  import { api, message } from "@/api/client";
  import type { DisplayStatus, Settings } from "@/types/settings";
  import Card from "@/components/ui/Card.svelte";
  import Field from "@/components/ui/Field.svelte";
  import Switch from "@/components/ui/Switch.svelte";
  import Select from "@/components/ui/Select.svelte";
  import Segmented from "@/components/ui/Segmented.svelte";

  let { display = $bindable() }: { display: Settings["display"] } = $props();

  const SCALING = [
    { label: "Fit", value: "fit" },
    { label: "Fill", value: "fill" },
  ];

  let status = $state<DisplayStatus | null>(null);
  let loadError = $state<string | null>(null);

  onMount(async () => {
    try {
      status = await api.display();
    } catch (e) {
      loadError = message(e);
    }
  });

  const hz = (r: number) =>
    Math.abs(r - Math.round(r)) < 0.02 ? String(Math.round(r)) : r.toFixed(2);

  const PRESETS = [
    {
      id: "1280x720@50.00",
      width: 1280,
      height: 720,
      refresh: 50,
      interlaced: false,
      label: "720p50",
    },
    {
      id: "1280x720@59.94",
      width: 1280,
      height: 720,
      refresh: 59.94,
      interlaced: false,
      label: "720p59.94",
    },
    {
      id: "1280x720@60.00",
      width: 1280,
      height: 720,
      refresh: 60,
      interlaced: false,
      label: "720p60",
    },
    {
      id: "1920x1080i@50.00",
      width: 1920,
      height: 1080,
      refresh: 50,
      interlaced: true,
      label: "1080i50",
    },
    {
      id: "1920x1080i@59.94",
      width: 1920,
      height: 1080,
      refresh: 59.94,
      interlaced: true,
      label: "1080i59.94",
    },
    {
      id: "1920x1080i@60.00",
      width: 1920,
      height: 1080,
      refresh: 60,
      interlaced: true,
      label: "1080i60",
    },
    {
      id: "1920x1080@23.98",
      width: 1920,
      height: 1080,
      refresh: 23.98,
      interlaced: false,
      label: "1080p23.98",
    },
    {
      id: "1920x1080@24.00",
      width: 1920,
      height: 1080,
      refresh: 24,
      interlaced: false,
      label: "1080p24",
    },
    {
      id: "1920x1080@25.00",
      width: 1920,
      height: 1080,
      refresh: 25,
      interlaced: false,
      label: "1080p25",
    },
    {
      id: "1920x1080@29.97",
      width: 1920,
      height: 1080,
      refresh: 29.97,
      interlaced: false,
      label: "1080p29.97",
    },
    {
      id: "1920x1080@30.00",
      width: 1920,
      height: 1080,
      refresh: 30,
      interlaced: false,
      label: "1080p30",
    },
    {
      id: "1920x1080@50.00",
      width: 1920,
      height: 1080,
      refresh: 50,
      interlaced: false,
      label: "1080p50",
    },
    {
      id: "1920x1080@59.94",
      width: 1920,
      height: 1080,
      refresh: 59.94,
      interlaced: false,
      label: "1080p59.94",
    },
    {
      id: "1920x1080@60.00",
      width: 1920,
      height: 1080,
      refresh: 60,
      interlaced: false,
      label: "1080p60",
    },
  ];

  const modes = $derived.by(() => {
    const detected = status?.modes ?? [];
    const used = new Set<string>();

    const presets = PRESETS.map((preset) => {
      const found = detected.find(
        (m) =>
          m.width === preset.width &&
          m.height === preset.height &&
          m.interlaced === preset.interlaced &&
          Math.abs(m.refresh - preset.refresh) < 0.04,
      );

      if (found) used.add(found.id);

      return {
        value: found?.id ?? preset.id,
        label: found
          ? preset.label
          : `${preset.label} (not supported by this screen)`,
        disabled: !found,
      };
    });

    const additional = detected
      .filter((m) => !used.has(m.id))
      .map((m) => {
        const notes = [
          m.preferred && "screen's choice",
          !m.fits_screen && "wrong shape, gets stretched",
          m.width >= 3840 && "may stutter",
          m.interlaced && "interlaced",
        ].filter(Boolean);

        return {
          value: m.id,
          label: `${m.width} × ${m.height}${m.interlaced ? "i" : "p"} · ${hz(m.refresh)} Hz${notes.length ? ` (${notes.join(", ")})` : ""}`,
          disabled: false,
        };
      });

    const list = [...presets, ...additional];

    // A mode saved while another screen was plugged in
    if (display.mode !== "auto" && !list.some((m) => m.value === display.mode))
      list.unshift({
        value: display.mode,
        label: `${display.mode} (not offered by this screen)`,
        disabled: true,
      });

    return [
      { value: "auto", label: "Automatic (best for video)", disabled: false },
      ...list,
    ];
  });

  const modeHelp = $derived(
    status?.error ??
      loadError ??
      "Automatic picks the sharpest mode up to 1080p, as close to 60 Hz as the screen goes. 4K is limited to 30 Hz.",
  );
</script>

<Card title="HDMI output">
  <Field
    label="Show video on HDMI"
    for="hdmi"
    help="Turn off when nothing is plugged in. Frees the decoder and some CPU for streaming."
  >
    <Switch id="hdmi" bind:checked={display.hdmi_output} />
  </Field>

  <Field
    label="Resolution"
    for="hdmi-mode"
    help={modeHelp}
    disabled={!display.hdmi_output}
  >
    <Select
      id="hdmi-mode"
      options={modes}
      bind:value={display.mode}
      disabled={!display.hdmi_output}
    />
  </Field>

  <Field label="Scaling" disabled={!display.hdmi_output}>
    <div class="w-full sm:w-64">
      <Segmented
        options={SCALING}
        value={display.scaling}
        disabled={!display.hdmi_output}
        onchange={(v) => (display.scaling = v)}
      />
    </div>
  </Field>

  <Field label="Rotate 180°" for="hdmi-rotate" disabled={!display.hdmi_output}>
    <Switch
      id="hdmi-rotate"
      bind:checked={display.rotate_180}
      disabled={!display.hdmi_output}
    />
  </Field>

  <Field label="Mirror" for="hdmi-mirror" disabled={!display.hdmi_output}>
    <Switch
      id="hdmi-mirror"
      bind:checked={display.mirror}
      disabled={!display.hdmi_output}
    />
  </Field>
</Card>
