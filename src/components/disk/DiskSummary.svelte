<script lang="ts">
    import { HardDrive } from "@lucide/svelte";
    import { formatLastScan } from "$types/disk";
    import { formatBytes } from "$utils/format";
    import type { DiskVolumeInfo } from "$types/disk";
    import MetricCard from "$components/MetricCard.svelte";

    let { volume }: { volume: DiskVolumeInfo | null } = $props();

    function diskAccent(percent: number) {
        if (percent < 70) return "#4ade80";
        if (percent < 90) return "#facc15";
        return "#f87171";
    }

    function diskAccentHover(percent: number) {
        if (percent < 70) return "#22c55e";
        if (percent < 90) return "#eab308";
        return "#ef4444";
    }
</script>

<div class="disk-summary">
    <div class="disk-top">
        <div
            class="disk-gauge"
            style="--disk-accent: {diskAccent(
                volume?.used_percent ?? 0,
            )}; --disk-accent-hover: {diskAccentHover(
                volume?.used_percent ?? 0,
            )};"
        >
            <MetricCard
                icon={HardDrive}
                label="Disk"
                value={volume?.used_percent ?? 0}
                displayValue={volume ? `${volume.used_percent}%` : "—"}
                tab="disk"
            />
        </div>
        <div class="disk-info-rows">
            <div class="info-row">
                <span class="info-label">Total</span>
                <span class="info-value"
                    >{formatBytes(volume?.total_bytes ?? 0)}</span
                >
            </div>
            <div class="info-row">
                <span class="info-label">Used</span>
                <span class="info-value"
                    >{formatBytes(volume?.used_bytes ?? 0)}</span
                >
            </div>
            <div class="info-row">
                <span class="info-label">Free</span>
                <span class="info-value"
                    >{formatBytes(volume?.available_bytes ?? 0)}</span
                >
            </div>
        </div>
    </div>
    <div class="disk-info-footer">
        <span class="footer-label">{volume?.label ?? "Disk"}</span>
        <span class="footer-sep">·</span>
        <span class="footer-label"
            >Last scan: {formatLastScan(volume?.last_scan_at)}</span
        >
    </div>
</div>

<style>
    .disk-summary {
        display: flex;
        flex-direction: column;
        padding: 12px 18px;
    }

    .disk-top {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 20px;
        padding-top: 4px;
    }

    .disk-gauge :global(.card) {
        width: 90px;
        height: 90px;
        border-radius: var(--radius-md);
        --color-accent: var(--disk-accent);
        --color-accent-hover: var(--disk-accent-hover);
    }

    .disk-info-rows {
        display: flex;
        flex-direction: column;
        justify-content: center;
        gap: 6px;
        min-width: 0;
    }

    .info-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 20px;
    }

    .info-label {
        font-size: 10px;
        font-weight: 500;
        color: var(--color-text-dim);
        text-transform: uppercase;
        letter-spacing: 0.04em;
        min-width: 40px;
    }

    .info-value {
        font-size: 12px;
        font-weight: 600;
        color: var(--color-text-primary);
        font-variant-numeric: tabular-nums;
    }

    .disk-info-footer {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        font-size: 11px;
        font-weight: 500;
        color: var(--color-text-secondary);
        margin-top: auto;
        padding-top: 10px;
        border-top: 1px solid var(--color-border-subtle);
    }

    .footer-sep {
        color: var(--color-text-dim);
    }
</style>
