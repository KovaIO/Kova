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
    import type { FlatProcess, Metrics, ProcessNode, Tab } from "./types";
    import {
        buildTree,
        cloneTree,
        matchesSearch,
        sortTreeBy,
    } from "./utils/process-tree";
    import { metricLabel } from "./utils/format";

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
    let cpuValue = 0;
    let ramValue = 0;
    let diskValue = 0;
    let networkBps = 0;
    let liveProcesses: FlatProcess[] = [];

    let unlisten: UnlistenFn;

    onMount(async () => {
        const snap = await invoke<Metrics | null>("get_current_metrics");
        if (snap) applyMetrics(snap);

        unlisten = await listen<Metrics>("metrics", (e) =>
            applyMetrics(e.payload),
        );
    });

    onDestroy(() => unlisten?.());

    function applyMetrics(m: Metrics) {
        cpuValue = m.cpu_percent;
        ramValue = m.ram_percent;
        diskValue = m.disk_percent;
        networkBps = m.network_bps;
        cpuHistory = m.cpu_history;
        ramHistory = m.ram_history;
        networkHistory = m.network_history;
        liveProcesses = m.processes;
    }

    function metricIsActive(p: ProcessNode): boolean {
        if (activeTab === "cpu") return p.cpu_percent > 0;
        if (activeTab === "ram") return p.ram_bytes > 0;
        if (activeTab === "network") return p.net_bps > 0;
        return false;
    }

    let search = "";
    let expandedPids = new Set<number>();

    function toggleExpand(pid: number) {
        const next = new Set(expandedPids);

        if (next.has(pid)) {
            next.delete(pid);
        } else {
            next.add(pid);
        }

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
        invoke("open_process", {
            pid,
            tab: activeTab,
        });
    }
</script>

<div class="page">
    <MetricGraph
        bind:activeTab
        {cpuHistory}
        {ramHistory}
        {networkHistory}
        {cpuValue}
        {ramValue}
        {diskValue}
        {networkBps}
    />

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
    {@const active = metricIsActive(proc)}

    <button
        class="process-row"
        class:clickable={hasKids}
        class:expandable={hasKids}
        style:padding-left="{16 + depth * 16}px"
        onclick={() => openProcess(proc.pid)}
    >
        <span
            class="chevron"
            aria-hidden="true"
            onclick={(e) => {
                e.stopPropagation();

                if (hasKids) {
                    toggleExpand(proc.pid);
                }
            }}
        >
            {#if hasKids}
                {#if open}
                    <ChevronDown size={11} />
                {:else}
                    <ChevronRight size={11} />
                {/if}
            {/if}
        </span>
        {#if proc.icon === "system"}
            <div class="proc-icon fallback-icon">
                <Cpu size={13} />
            </div>
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

        <span class="proc-name" class:active>{proc.name}</span>

        {#if hasKids}
            <span class="child-badge">{proc.children.length}</span>
        {/if}

        <span class="proc-metric" class:active
            >{metricLabel(proc, activeTab)}</span
        >
    </button>

    {#if hasKids && open}
        {#each proc.children as child (`${child.pid}-${activeTab}`)}
            {@render row(child, depth + 1)}
        {/each}
    {/if}
{/snippet}

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
        background: rgba(255, 255, 255, 0.1);
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

        transition: background 0.12s;
    }

    .process-row:focus {
        outline: none;
    }

    .process-row:focus-visible {
        background: var(--color-button-bg-hover);
    }

    .chevron {
        width: 14px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        color: var(--color-text-dim);
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

        background: rgba(255, 255, 255, 0.08);
        color: var(--color-text-secondary);

        line-height: 0;
    }
    .fallback-icon :global(svg) {
        transform: translate(-0.5px, -0.5px);
    }

    .proc-name {
        flex: 1;
        font-size: 14px;
        font-weight: 500;
        color: var(--color-text-dim);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .proc-name.active {
        color: var(--color-text-secondary);
    }

    .child-badge {
        font-size: 12px;
        color: var(--color-text-dim);
        background: rgba(255, 255, 255, 0.07);
        border-radius: 99px;
        padding: 1px 6px;
        flex-shrink: 0;
    }

    .proc-metric {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-dim);
        letter-spacing: 0.02em;
        flex-shrink: 0;
        min-width: 60px;
        text-align: right;
        font-variant-numeric: tabular-nums;
    }

    .proc-metric.active {
        color: var(--color-text-secondary);
    }

    .empty {
        text-align: center;
        color: var(--color-text-dim);
        font-size: 12px;
        padding: 24px 0;
    }
</style>
