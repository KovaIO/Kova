<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import MetricCard from "$components/MetricCard.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";

    interface Metrics {
        cpu_percent: number;
        ram_used: number;
        ram_total: number;
        ram_percent: number;
        disk_used: number;
        disk_total: number;
        disk_percent: number;
    }

    let cpu = 0;
    let ram = 0;
    let disk = 0;
    let cpuDisplay = "—";
    let ramDisplay = "—";
    let diskDisplay = "—";

    function formatBytes(bytes: number): string {
        const gb = bytes / 1024 / 1024 / 1024;
        return gb >= 1
            ? `${gb.toFixed(1)} GB`
            : `${(bytes / 1024 / 1024).toFixed(0)} MB`;
    }

    function applyMetrics(m: Metrics) {
        cpu = Math.round(m.cpu_percent);
        ram = Math.round(m.ram_percent);
        disk = Math.round(m.disk_percent);
        cpuDisplay = `${cpu}%`;
        ramDisplay = formatBytes(m.ram_used);
        diskDisplay = `${disk}%`;
    }

    let unlisten: UnlistenFn;

    onMount(async () => {
        unlisten = await listen<Metrics>("metrics", (event) => {
            applyMetrics(event.payload);
        });
    });

    onDestroy(() => unlisten?.());

    let windowManager = false;
    let clipboardHistory = false;
    let brightness = 80;

    async function openPreferences() {
        await invoke("open_preferences");
    }
</script>

<div class="page">
    <div class="shell">
        <section class="metrics">
            <MetricCard
                icon="󰻠"
                label="CPU"
                value={cpu}
                displayValue={cpuDisplay}
            />
            <MetricCard
                icon="󰍛"
                label="RAM"
                value={ram}
                displayValue={ramDisplay}
            />
            <MetricCard
                icon="󰋊"
                label="Disk"
                value={disk}
                displayValue={diskDisplay}
            />
        </section>

        <div class="divider"></div>

        <section class="switches">
            <Toggle
                id="wm"
                label="Window Manager"
                bind:checked={windowManager}
            />
            <Toggle
                id="cb"
                label="Clipboard History"
                bind:checked={clipboardHistory}
            />
        </section>

        <div class="divider"></div>

        <section class="brightness-section">
            <BrightnessSlider bind:value={brightness} />
        </section>
    </div>

    <button class="prefs-btn" type="button" on:click={openPreferences}
        >Preferences</button
    >
</div>

<style>
    .page {
        display: flex;
        flex-direction: column;
        gap: 8px;
        padding: 0;
        background: transparent;
    }

    .shell {
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border-radius: var(--radius-md);
        border: 2px solid var(--color-border-subtle);
        display: flex;
        flex-direction: column;
        padding: 20px 18px;
        gap: 16px;
        color: white;
    }

    .metrics {
        display: flex;
        gap: 10px;
        justify-content: center;
    }

    .divider {
        height: 1px;
        background: var(--color-border-subtle);
        flex-shrink: 0;
        margin-left: -18px;
        margin-right: -18px;
    }

    .switches {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .brightness-section {
        padding: 0 2px;
    }

    .prefs-btn {
        width: 100%;
        padding: 12px 18px;
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        color: var(--color-text-tertiary);
        font-size: 12.5px;
        font-weight: 500;
        font-family: inherit;
        letter-spacing: 0.02em;
        text-align: left;
        cursor: pointer;
        transition: var(--transition-medium);
    }

    .prefs-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-secondary);
    }
</style>
