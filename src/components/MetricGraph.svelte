<script lang="ts">
    export let activeTab: "cpu" | "ram" | "disk";
    export let cpuHistory: number[];
    export let ramHistory: number[];
    export let diskHistory: number[];
    export let cpuValue: number;
    export let ramValue: number;
    export let diskValue: number;

    $: history =
        activeTab === "cpu" ? cpuHistory
        : activeTab === "ram" ? ramHistory
        : diskHistory;

    $: currentValue =
        activeTab === "cpu" ? cpuValue
        : activeTab === "ram" ? ramValue
        : diskValue;

    function buildPath(data: number[]): string {
        const step = 100 / (data.length - 1);
        return data.map((v, i) => {
            const x = i * step;
            const y = 100 - v;
            return `${i === 0 ? "M" : "L"} ${x.toFixed(2)} ${y.toFixed(2)}`;
        }).join(" ");
    }

    function buildFill(data: number[]): string {
        return `${buildPath(data)} L 100 100 L 0 100 Z`;
    }

    $: linePath = buildPath(history);
    $: fillPath = buildFill(history);
</script>

<div class="card">
    <div class="tabs">
        <button class="tab" class:active={activeTab === "cpu"}  on:click={() => (activeTab = "cpu")}>CPU</button>
        <button class="tab" class:active={activeTab === "ram"}  on:click={() => (activeTab = "ram")}>Memory</button>
        <button class="tab" class:active={activeTab === "disk"} on:click={() => (activeTab = "disk")}>Disk</button>
    </div>

    <div class="graph-wrap">
        <div class="peak-label">{currentValue}%</div>
        <svg viewBox="0 0 100 100" preserveAspectRatio="none" class="graph-svg">
            <defs>
                <linearGradient id="fill-grad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%"   stop-color="var(--color-accent-strong)" />
                    <stop offset="100%" stop-color="var(--color-accent)" stop-opacity="0" />
                </linearGradient>
            </defs>
            <path d={fillPath} fill="url(#fill-grad)" />
            <path
                d={linePath}
                fill="none"
                stroke="var(--color-accent)"
                stroke-width="1.5"
                stroke-linejoin="round"
                stroke-linecap="round"
                vector-effect="non-scaling-stroke"
            />
        </svg>
    </div>
</div>

<style>
    .card {
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        color: white;
        overflow: hidden;
        flex-shrink: 0;
        padding: 16px;
    }

    .tabs {
        display: flex;
        gap: 4px;
        margin-bottom: 14px;
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
        cursor: pointer;
        transition: var(--transition-fast);
    }

    .tab.active {
        background: var(--color-accent-soft);
        color: var(--color-accent);
    }

    .peak-label {
        font-size: 11px;
        font-weight: 500;
        color: var(--color-text-muted);
        margin-bottom: 6px;
        letter-spacing: 0.02em;
    }

    .graph-svg {
        display: block;
        width: 100%;
        aspect-ratio: 2 / 1;
        border-radius: 6px;
    }
</style>