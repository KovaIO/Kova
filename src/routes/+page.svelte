<script lang="ts">
    import MetricCard from "$components/MetricCard.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";

    let cpu = 34;
    let ram = 61;
    let disk = 72;

    let cpuDisplay = `${cpu}%`;
    let ramDisplay = `6.1 GB`;
    let diskDisplay = `${disk}%`;

    let windowManager = false;
    let clipboardHistory = false;
    let brightness = 80;
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

    <button class="prefs-btn" type="button">Preferences</button>
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
        border: 1px solid var(--color-border-subtle);
        display: flex;
        flex-direction: column;
        padding: 20px 18px;
        gap: 16px;
        color: white;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
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
        border: 1px solid var(--color-border-subtle);
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
