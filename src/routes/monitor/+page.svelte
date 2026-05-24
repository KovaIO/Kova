<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";
    import { page } from "$app/state";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import MetricGraph from "$components/MetricGraph.svelte";
    import { Search } from "@lucide/svelte";

    type Tab = "cpu" | "ram" | "disk";
    let activeTab: Tab = "cpu";

    $: {
        const t = page.url.searchParams.get("tab");
        if (t === "cpu" || t === "ram" || t === "disk") {
            activeTab = t;
        }
    }

    interface Metrics {
        cpu_percent: number;
        ram_used: number;
        ram_total: number;
        ram_percent: number;
        disk_used: number;
        disk_total: number;
        disk_percent: number;
        cpu_history: number[];
        ram_history: number[];
    }

    let cpuHistory: number[] = [];
    let ramHistory: number[] = [];
    let diskHistory: number[] = [];

    let cpuValue = 0;
    let ramValue = 0;
    let diskValue = 0;

    let unlisten: UnlistenFn;

    onMount(async () => {
        const snapshot = await invoke<Metrics | null>("get_current_metrics");
        if (snapshot) {
            cpuValue   = snapshot.cpu_percent;
            ramValue   = snapshot.ram_percent;
            diskValue  = snapshot.disk_percent;
            cpuHistory = snapshot.cpu_history;
            ramHistory = snapshot.ram_history;
        }
        
        unlisten = await listen<Metrics>("metrics", (event) => {
            const m = event.payload;
            cpuValue   = m.cpu_percent;
            ramValue   = m.ram_percent;
            diskValue  = m.disk_percent;
            cpuHistory = m.cpu_history;
            ramHistory = m.ram_history;
        });
    });

    onDestroy(() => unlisten?.());

    let search = "";

    interface Process {
        name: string;
        icon: string;
        cpu: number;
        ram: number;
        disk: number;
    }

    let processes: Process[] = [];

    $: sorted = [...processes].sort((a, b) =>
        activeTab === "cpu"
            ? b.cpu - a.cpu
            : activeTab === "ram"
              ? b.ram - a.ram
              : b.disk - a.disk,
    );

    $: filtered = sorted.filter((p) =>
        p.name.toLowerCase().includes(search.toLowerCase()),
    );

    function metricLabel(p: Process): string {
        if (activeTab === "cpu") return `${p.cpu.toFixed(1)}%`;
        if (activeTab === "ram") return `${p.ram.toFixed(1)} MB`;
        return `${p.disk.toFixed(1)} MB/s`;
    }
</script>

<div class="page">
    <MetricGraph
        {activeTab}
        {cpuHistory}
        {ramHistory}
        {diskHistory}
        {cpuValue}
        {ramValue}
        {diskValue}
    />

    <div class="card search-card">
        <Search class="search-icon" size={14} />
        <input
            class="search-input"
            type="text"
            placeholder="Search process"
            bind:value={search}
        />
    </div>

    <div class="card list-card">
        {#each filtered as p (p.name)}
            <div class="process-row">
                <span class="process-icon">{p.icon}</span>
                <span class="process-name">{p.name}</span>
                <span class="process-metric">{metricLabel(p)}</span>
            </div>
        {:else}
            <p class="empty">No processes found</p>
        {/each}
    </div>
</div>

<style>
    :global(html, body) {
        margin: 0;
        padding: 0;
        background: transparent !important;
        overflow: hidden;
    }

    .page {
        display: flex;
        flex-direction: column;
        gap: 8px;
        background: transparent;
        height: 100vh;
        overflow: hidden;
    }

    .card {
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        color: white;
        overflow: hidden;
    }

    .search-card {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 0 14px;
        height: 42px;
        flex-shrink: 0;
    }

    .search-input {
        flex: 1;
        background: transparent;
        border: none;
        outline: none;
        color: var(--color-text-secondary);
        font-size: 12.5px;
        font-family: inherit;
    }

    .search-input::placeholder {
        color: var(--color-text-dim);
    }

    .list-card {
        flex: 1;
        overflow-y: auto;
        padding: 6px 0;
        min-height: 0;
    }

    .list-card::-webkit-scrollbar {
        width: 3px;
    }
    .list-card::-webkit-scrollbar-track {
        background: transparent;
    }
    .list-card::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 999px;
    }

    .process-row {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 16px;
        transition: var(--transition-fast);
    }

    .process-row:hover {
        background: var(--color-button-bg-hover);
    }

    .process-icon {
        font-size: 16px;
        width: 22px;
        text-align: center;
        flex-shrink: 0;
    }

    .process-name {
        flex: 1;
        font-size: 12.5px;
        font-weight: 450;
        color: var(--color-text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .process-metric {
        font-size: 12px;
        font-weight: 500;
        color: var(--color-text-muted);
        letter-spacing: 0.01em;
        flex-shrink: 0;
    }

    .empty {
        text-align: center;
        color: var(--color-text-dim);
        font-size: 12px;
        padding: 24px 0;
    }
</style>
