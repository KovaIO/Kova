<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";
    import { page } from "$app/state";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import MetricGraph from "$components/MetricGraph.svelte";
    import {
        Search,
        ChevronRight,
        ChevronDown,
        Terminal,
        Cpu,
    } from "@lucide/svelte";
    import type {
        FlatProcess,
        Metrics,
        ProcessNode,
        Tab,
    } from "$types/metrics";
    import {
        buildTree,
        cloneTree,
        matchesSearch,
        sortTreeBy,
    } from "$utils/process-tree";
    import { metricLabel } from "$utils/format";

    let activeTab: Tab = "cpu";

    $: {
        const tab = page.url.searchParams.get("tab");
        if (
            tab === "cpu" ||
            tab === "ram" ||
            tab === "disk" ||
            tab === "network"
        ) {
            activeTab = tab;
        }
    }

    let cpuHistory: number[] = [];
    let ramHistory: number[] = [];
    let networkHistory: number[] = [];
    let ramTotal = 0;
    let networkPeak = 0;

    let liveProcesses: FlatProcess[] = [];

    let selectedHistoryIndex: number | null = null;
    let historyMode = false;

    let unlisten: UnlistenFn;

    onMount(async () => {
        const snap = await invoke<Metrics | null>("get_current_metrics");
        if (snap) applyMetrics(snap);
        unlisten = await listen<Metrics>("metrics", async (e) => {
            if (!historyMode) {
                applyMetrics(e.payload);
                return;
            }

            if (selectedHistoryIndex !== null) {
                selectedHistoryIndex -= 1;

                const snap = await invoke<Metrics>("get_snapshot", {
                    index: selectedHistoryIndex,
                });

                applyMetrics(snap);
            }
        });
    });

    onDestroy(() => unlisten?.());

    async function handleBarClick(index: number) {
        if (selectedHistoryIndex === index) {
            exitHistoryMode();
            return;
        }

        historyMode = true;
        selectedHistoryIndex = index;

        const snap = await invoke<Metrics>("get_snapshot", { index });

        applyMetrics(snap);
    }

    function exitHistoryMode() {
        historyMode = false;
        selectedHistoryIndex = null;
    }

    function applyMetrics(m: Metrics) {
        liveProcesses = m.processes;

        if (m.ram_total > 0) ramTotal = m.ram_total;
        if (m.network_bps > networkPeak) networkPeak = m.network_bps;

        cpuHistory = m.cpu_history;
        networkHistory = m.network_history;

        ramHistory = m.ram_history.map((pct) => (pct / 100) * m.ram_total);
    }

    let search = "";
    let expandedPids = new Set<number>();

    function toggleExpand(pid: number) {
        const next = new Set(expandedPids);
        next.has(pid) ? next.delete(pid) : next.add(pid);
        expandedPids = next;
    }

    $: searchLower = search.trim().toLowerCase();

    let cpuTree: ProcessNode[] = [];
    let ramTree: ProcessNode[] = [];
    let networkTree: ProcessNode[] = [];

    $: {
        const { roots } = buildTree(liveProcesses);
        cpuTree = cloneTree(roots);
        sortTreeBy(cpuTree, "cpu");
        ramTree = cloneTree(roots);
        sortTreeBy(ramTree, "ram");
        networkTree = cloneTree(roots);
        sortTreeBy(networkTree, "network");
    }

    $: processTree =
        activeTab === "cpu"
            ? cpuTree
            : activeTab === "ram"
              ? ramTree
              : networkTree;

    $: displayProcesses = processTree.filter((p) =>
        matchesSearch(p, searchLower),
    );
    $: showProcessList = activeTab !== "disk";

    function openProcess(pid: number) {
        invoke("open_process", { pid, tab: activeTab });
    }
</script>

<div class="page">
    <div class="graph">
        <MetricGraph
            bind:activeTab
            {cpuHistory}
            {ramHistory}
            {networkHistory}
            ramMax={ramTotal}
            netMax={networkPeak || undefined}
            onBarClick={handleBarClick}
            selectedIndex={selectedHistoryIndex}
            onBackgroundClick={exitHistoryMode}
        />
    </div>

    {#if showProcessList}
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
            {#if displayProcesses.length === 0}
                <p class="empty">
                    {search ? `No results for "${search}"` : "No data yet"}
                </p>
            {:else}
                {#each displayProcesses as proc (`${proc.pid}-${activeTab}`)}
                    {@render row(proc, 0)}
                {/each}
            {/if}
        </div>
    {/if}
</div>

{#snippet row(proc: ProcessNode, depth: number)}
    {@const hasKids = proc.children.length > 0}
    {@const open = expandedPids.has(proc.pid)}

    <div class="process-row" style:padding-left="{16 + depth * 16}px">
        <button
            type="button"
            class="chevron"
            aria-label={open ? "Collapse" : "Expand"}
            onclick={() => {
                if (hasKids) toggleExpand(proc.pid);
            }}
        >
            {#if hasKids}
                {#if open}<ChevronDown
                        size={14}
                        strokeWidth={3}
                    />{:else}<ChevronRight size={14} strokeWidth={3} />{/if}
            {/if}
        </button>
        <button
            type="button"
            class="process-main"
            onclick={() => openProcess(proc.pid)}
        >
            {#if proc.icon === "system"}
                <div class="proc-icon fallback-icon"><Cpu size={13} /></div>
            {:else if proc.icon === "terminal"}
                <div class="proc-icon fallback-icon">
                    <Terminal size={13} />
                </div>
            {:else if proc.icon}
                <img
                    class="proc-icon"
                    src={`data:image/png;base64,${proc.icon}`}
                    alt=""
                />
            {:else}
                <div class="proc-icon fallback"></div>
            {/if}
            <div class="proc-main-info">
                <span class="proc-name">{proc.name}</span>
                {#if hasKids}
                    <span class="child-badge">
                        <span class="plus">+</span>
                        <span>{proc.children.length}</span>
                    </span>
                {/if}
            </div>
            <span class="proc-metric">{metricLabel(proc, activeTab)}</span>
        </button>
    </div>

    {#if hasKids && open}
        {#each proc.children as child (`${child.pid}-${activeTab}`)}
            {@render row(child, depth + 1)}
        {/each}
    {/if}
{/snippet}

<style>
    .graph {
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
    }

    .page {
        display: flex;
        flex-direction: column;
        gap: 8px;
        height: 100vh;
        overflow: hidden;
        background: transparent;
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
        contain: strict;
        flex: 1;
        overflow-y: auto;
        padding: 6px 0;
        min-height: 0;
        overscroll-behavior: contain;
        will-change: scroll-position;
    }

    .list-card::-webkit-scrollbar {
        width: 3px;
    }
    .list-card::-webkit-scrollbar-track {
        background: transparent;
    }
    .list-card::-webkit-scrollbar-thumb {
        background: var(--color-border-strong);
        border-radius: 999px;
    }

    .process-row {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        min-height: 40px;
        box-sizing: border-box;
        border: none;
        background: transparent;
        font: inherit;
        text-align: left;
        padding-top: 7px;
        padding-bottom: 7px;
        padding-right: 16px;
        transition: var(--transition-fast);
    }

    .process-row:focus {
        outline: none;
    }
    .process-row:focus-visible {
        background: var(--color-button-bg-hover);
    }

    .chevron {
        width: 26px;
        flex-shrink: 0;
        border: none;
        background: transparent;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--color-text-tertiary);
    }

    .process-main {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 8px;
        box-sizing: border-box;
        border: none;
        background: transparent;
        font: inherit;
        text-align: left;
    }

    .proc-icon {
        width: 20px;
        height: 20px;
        flex-shrink: 0;
        border-radius: 4px;
        object-fit: contain;
    }

    img.proc-icon {
        object-fit: contain;
    }

    .fallback-icon {
        display: grid;
        place-items: center;
        background: var(--color-track-fill);
        color: var(--color-text-secondary);
        line-height: 0;
    }

    .fallback-icon :global(svg) {
        transform: translate(-0.5px, -0.5px);
    }

    .proc-main-info {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 8px;
        min-width: 0;
    }

    .proc-name {
        font-size: 14px;
        font-weight: 500;
        color: var(--color-text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .child-badge {
        display: inline-flex;
        align-items: center;
        gap: 3px;
        font-size: 12px;
        color: var(--color-text-secondary);
        background: var(--color-track-fill);
        border-radius: var(--radius-md);
        padding: 1px 6px;
        flex-shrink: 0;
    }

    .plus {
        position: relative;
        top: -0.5px;
        font-size: 11px;
        font-weight: 600;
    }

    .proc-metric {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-secondary);
        letter-spacing: 0.02em;
        flex-shrink: 0;
        min-width: 60px;
        text-align: right;
        font-variant-numeric: tabular-nums;
    }

    .empty {
        text-align: center;
        color: var(--color-text-dim);
        font-size: 12px;
        padding: 24px 0;
    }
</style>
