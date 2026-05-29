<script lang="ts">
    import { formatBps, formatBytes } from "$utils/format";

    type Tab = "cpu" | "ram" | "disk" | "network";

    export let activeTab: Tab;
    export let showDiskTab = true;

    export let cpuHistory: number[];
    export let ramHistory: number[];
    export let networkHistory: number[];

    export let ramMax: number | null = null;
    export let netMax: number | null = null;

    export let selectedIndex: number | null = null;
    export let onBarClick: ((index: number) => void) | undefined;
    export let onBackgroundClick: (() => void) | undefined;

    $: networkMax = Math.max(...networkHistory, 1);
    $: ramTotalBytes = ramMax ?? Math.max(...ramHistory, 1);

    $: history = (() => {
        if (activeTab === "cpu") return cpuHistory;
        if (activeTab === "ram")
            return ramHistory.map((v) => (v / ramTotalBytes) * 100);
        return networkHistory.map((v) => (v / networkMax) * 100);
    })();

    $: yLabels = (() => {
        if (activeTab === "ram") {
            return [1, 0.75, 0.5, 0.25].map((r) =>
                formatBytes(ramTotalBytes * r),
            );
        }
        if (activeTab === "network") {
            const peak = netMax ?? networkMax;
            return [1, 0.75, 0.5, 0.25].map((r) => formatBps(peak * r));
        }
        return [1, 0.75, 0.5, 0.25].map((r) => `${Math.round(100 * r)}%`);
    })();

    const SPLIT = 0.3;
</script>

<div class="card">
    <div class="tabs">
        <button
            class="tab"
            class:active={activeTab === "cpu"}
            onclick={() => (activeTab = "cpu")}>CPU</button
        >
        <button
            class="tab"
            class:active={activeTab === "ram"}
            onclick={() => (activeTab = "ram")}>Memory</button
        >
        <button
            class="tab"
            class:active={activeTab === "network"}
            onclick={() => (activeTab = "network")}>Network</button
        >
        {#if showDiskTab}
            <button
                class="tab"
                class:active={activeTab === "disk"}
                onclick={() => (activeTab = "disk")}>Disk</button
            >
        {/if}
    </div>

    <div class="graph-area">
        <div class="chart-wrap">
            <div class="grid-lines">
                {#each [0, 1, 2, 3] as i}
                    <div class="grid-line">
                        <span class="grid-label">{yLabels[i]}</span>
                    </div>
                {/each}
            </div>

            <div
                role="button"
                class="bars"
                tabindex="0"
                onclick={(e) => {
                    if (e.target === e.currentTarget) {
                        onBackgroundClick?.();
                    }
                }}
                onkeydown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                        onBackgroundClick?.();
                    }
                }}
            >
                {#each history as value, i}
                    {@const pct = Math.max(value, 1)}
                    {@const topPct = Math.min(pct * SPLIT, pct)}
                    {@const bottomPct = pct - topPct}
                    <button
                        class="bar-wrap"
                        class:selected={selectedIndex === i}
                        class:dimmed={selectedIndex !== null &&
                            selectedIndex !== i}
                        aria-label="bar"
                        style="height: {pct}%"
                        onclick={(e) => {
                            e.stopPropagation();
                            onBarClick?.(i);
                        }}
                    >
                        <div
                            class="bar-top"
                            style="height: {(topPct / pct) * 100}%"
                        ></div>
                        <div
                            class="bar-bottom"
                            style="height: {(bottomPct / pct) * 100}%"
                        ></div>
                    </button>
                {/each}
            </div>
        </div>
    </div>
</div>

<style>
    .card {
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border-radius: var(--radius-md);
        color: white;
        overflow: hidden;
        flex-shrink: 0;
        padding: 16px;
    }

    .tabs {
        display: flex;
        gap: 4px;
        margin-bottom: 16px;
        justify-content: center;
    }

    .tab {
        padding: 5px 14px;
        border-radius: 999px;
        border: none;
        background: var(--color-track-bg);
        color: var(--color-text-dim);
        font-size: 12px;
        font-weight: 500;
        font-family: inherit;
        transition: var(--transition-fast);
    }

    .tab.active {
        background: var(--color-accent-soft);
        color: var(--color-accent);
    }

    .graph-area {
        display: flex;
    }

    .chart-wrap {
        position: relative;
        flex: 1;
        height: 140px;
    }

    .grid-lines {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        pointer-events: none;
        z-index: 2;
    }

    .grid-line {
        width: 100%;
        height: 1px;
        background: rgba(255, 255, 255, 0.05);
        position: relative;
        display: flex;
        align-items: flex-start;
    }

    .grid-label {
        font-size: 10px;
        font-weight: 500;
        color: var(--color-text-secondary);
        letter-spacing: 0.02em;
        line-height: 1;
        white-space: nowrap;
        padding: 2px 5px;
        background: var(--color-surface-elevated);
        border-radius: 5px;
        transform: translateY(-100%);
        margin-left: 2px;
    }

    .bars {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: flex-end;
        justify-content: flex-end;
        gap: 2px;
        padding: 0 1px;
        z-index: 1;
        overflow: hidden;
    }

    .bar-wrap {
        flex: 1;
        max-width: 8px;
        min-width: 2px;
        display: flex;
        flex-direction: column;
        border-radius: 2px 2px 0 0;
        overflow: hidden;
        border: none;
        padding: 0;
        background: transparent;
        transition: height 300ms ease;
    }
    .bar-wrap.dimmed {
        opacity: 0.28;
        filter: grayscale(1);
    }

    .bar-wrap.selected {
        opacity: 1;
        filter: none;
    }

    .bar-top {
        background: var(--color-accent);
        flex-shrink: 0;
    }

    .bar-bottom {
        background: var(--color-accent-soft);
        flex: 1;
    }
</style>
