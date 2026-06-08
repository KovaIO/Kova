<script lang="ts">
    import { HardDrive } from "@lucide/svelte";
    import { formatLastScan } from "$types/disk";
    import { formatBytes } from "$utils/format";
    import type { DiskVolumeInfo } from "$types/disk";

    let { volume }: { volume: DiskVolumeInfo | null } = $props();

    function diskColor(percent: number) {
        if (percent < 70) return "#4ade80";
        if (percent < 90) return "#facc15";
        return "#f87171";
    }

    function diskColorHover(percent: number) {
        if (percent < 70) return "#22c55e";
        if (percent < 90) return "#eab308";
        return "#ef4444";
    }
</script>

<div class="disk-summary">
    <div class="disk-top">
        <div class="disk-gauge">
            <div class="disk-gauge-track">
                <div
                    class="disk-gauge-fill"
                    style="height: {volume?.used_percent ?? 0}%; background: linear-gradient(to top, {diskColor(volume?.used_percent ?? 0)}, {diskColorHover(volume?.used_percent ?? 0)});"
                ></div>
            </div>
            <div class="disk-gauge-label">
                <HardDrive size={20} />
                <span class="disk-gauge-value">{volume?.used_percent ?? 0}%</span>
            </div>
        </div>
        <div class="disk-info-rows">
            <div class="info-row">
                <span class="info-label">Total</span>
                <span class="info-value">{formatBytes(volume?.total_bytes ?? 0)}</span>
            </div>
            <div class="info-row">
                <span class="info-label">Used</span>
                <span class="info-value">{formatBytes(volume?.used_bytes ?? 0)}</span>
            </div>
            <div class="info-row">
                <span class="info-label">Free</span>
                <span class="info-value">{formatBytes(volume?.available_bytes ?? 0)}</span>
            </div>
        </div>
    </div>
    <div class="disk-info-footer">
        <span class="footer-label">{volume?.label ?? "Disk"}</span>
        <span class="footer-sep">·</span>
        <span class="footer-label">Last scan: {formatLastScan(volume?.last_scan_at)}</span>
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

    .disk-gauge {
        position: relative;
        width: 90px;
        height: 90px;
        flex-shrink: 0;
        border-radius: var(--radius-md);
        background: var(--color-track-fill);
        overflow: hidden;
        display: flex;
        align-items: flex-end;
    }

    .disk-gauge-track {
        position: absolute;
        inset: 0;
    }

    .disk-gauge-fill {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        border-radius: 0 0 var(--radius-md) var(--radius-md);
        opacity: 0.85;
        transition: height 600ms cubic-bezier(0.4, 0, 0.2, 1);
    }

    .disk-gauge-label {
        position: relative;
        z-index: 1;
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 2px;
        color: var(--color-text-secondary);
    }

    .disk-gauge-value {
        font-size: 13px;
        font-weight: 700;
        color: var(--color-text-primary);
        font-variant-numeric: tabular-nums;
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
