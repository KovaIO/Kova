<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";
    import { page } from "$app/state";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import MetricGraph from "$components/MetricGraph.svelte";
    import { canUse, license } from "$stores/license";
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
    import WindowAnimation from "$components/WindowAnimation.svelte";
    import DiskPanel from "$components/disk/DiskPanel.svelte";
    import DiskSummary from "$components/disk/DiskSummary.svelte";
    import { fetchDiskVolumeInfo } from "$services/disk";
    import type { DiskVolumeInfo } from "$types/disk";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

    let activeTab: Tab = "cpu";
    let previousTab: Tab = "cpu";

    let diskVolume: DiskVolumeInfo | null = null;
    let diskKey = 0;

    onMount(async () => {
        diskVolume = await fetchDiskVolumeInfo();
    });

    $: if (previousTab === "disk" && activeTab !== "disk") {
        // clear disk scan result
        diskKey = (diskKey + 1) % 1000;
    }
    $: previousTab = activeTab;

    const unlistenTab = getCurrentWebviewWindow().listen<string>(
        "set-tab",
        (e) => {
            activeTab = e.payload as Tab;
        },
    );

    let cpuHistory: number[] = [];
    let ramHistory: number[] = [];
    let networkHistory: number[] = [];
    let ramTotal = 0;
    let networkPeak = 0;

    let liveProcesses: FlatProcess[] = [];

    let selectedHistoryIndex: number | null = null;
    let historyMode = false;
    let prevHistoryLength = 0;

    let unlisten: UnlistenFn;

    onMount(async () => {
        unlisten = await listen<Metrics>("metrics", (e) => {
            if (!historyMode) {
                applyLiveMetrics(e.payload);
                return;
            }

            updateGraph(e.payload);

            if (selectedHistoryIndex !== null) {
                const history =
                    activeTab === "ram"
                        ? ramHistory
                        : activeTab === "network"
                          ? networkHistory
                          : cpuHistory;
                if (history.length === prevHistoryLength) {
                    selectedHistoryIndex--;
                }
                if (
                    history.length === 0 ||
                    selectedHistoryIndex < 0 ||
                    selectedHistoryIndex >= history.length
                ) {
                    selectedHistoryIndex = null;
                }
            }
            prevHistoryLength = (
                activeTab === "ram"
                    ? ramHistory
                    : activeTab === "network"
                      ? networkHistory
                      : cpuHistory
            ).length;
        });

        let snap = await invoke<Metrics | null>("get_current_metrics");
        while (!snap) {
            await new Promise((r) => setTimeout(r, 100));
            snap = await invoke<Metrics | null>("get_current_metrics");
        }
        applyLiveMetrics(snap);
    });

    onDestroy(async () => {
        unlisten?.();
        (await unlistenTab)();
    });

    async function handleBarClick(index: number) {
        if (selectedHistoryIndex === index) {
            exitHistoryMode();
            return;
        }

        historyMode = true;
        selectedHistoryIndex = index;

        const history =
            activeTab === "ram"
                ? ramHistory
                : activeTab === "network"
                  ? networkHistory
                  : cpuHistory;
        prevHistoryLength = history.length;

        const snap = await invoke<Metrics>("get_snapshot", { index });

        liveProcesses = snap.processes;
    }

    async function exitHistoryMode() {
        historyMode = false;
        selectedHistoryIndex = null;

        const snap = await invoke<Metrics | null>("get_current_metrics");
        if (snap) applyLiveMetrics(snap);
    }

    function updateGraph(m: Metrics) {
        if (m.ram_total > 0) ramTotal = m.ram_total;
        if (m.network_bps > networkPeak) networkPeak = m.network_bps;

        cpuHistory = m.cpu_history;
        networkHistory = m.network_history;
        ramHistory = m.ram_history.map((pct) => (pct / 100) * m.ram_total);
    }

    function applyLiveMetrics(m: Metrics) {
        liveProcesses = m.processes;
        updateGraph(m);
    }

    let search = "";
    let expandedPids = new Set<number>();

    function toggleExpand(pid: number) {
        const next = new Set(expandedPids);
        next.has(pid) ? next.delete(pid) : next.add(pid);
        expandedPids = next;
    }

    $: searchLower = search.trim().toLowerCase();

    let processTree: ProcessNode[] = [];

    $: if (activeTab !== "disk") {
        const sortKey =
            activeTab === "network"
                ? "network"
                : activeTab === "ram"
                  ? "ram"
                  : "cpu";
        const { roots } = buildTree(liveProcesses);
        processTree = cloneTree(roots);
        sortTreeBy(processTree, sortKey);
    } else {
        processTree = [];
    }

    $: displayProcesses = processTree.filter((p) =>
        matchesSearch(p, searchLower),
    );
    $: diskCleanEnabled = canUse("disk_clean", $license);
    $: showProcessList = activeTab !== "disk";
    $: if (activeTab === "disk" && !diskCleanEnabled) {
        activeTab = "cpu";
    }

    function openProcess(pid: number) {
        invoke("open_process", { pid, tab: activeTab });
    }
</script>

<WindowAnimation>
    <div class="page">
        <div class="graph" class:graph-compact={activeTab === "disk"}>
            <MetricGraph
                bind:activeTab
                {diskCleanEnabled}
                showDiskTab={true}
                hideChart={activeTab === "disk"}
                {cpuHistory}
                {ramHistory}
                {networkHistory}
                ramMax={ramTotal}
                netMax={networkPeak || undefined}
                onBarClick={handleBarClick}
                selectedIndex={selectedHistoryIndex}
                onBackgroundClick={exitHistoryMode}
            />
            {#if activeTab === "disk"}
                <DiskSummary volume={diskVolume} />
            {/if}
        </div>

        {#if activeTab === "disk"}
            {#key diskKey}
                <DiskPanel bind:volume={diskVolume} />
            {/key}
        {:else if showProcessList}
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
</WindowAnimation>

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
    .graph-compact {
        padding-top: 8px;
        padding-bottom: 0;
        height: 232px;
        display: flex;
        flex-direction: column;
    }

    .graph-compact :global(.disk-summary) {
        flex: 1;
        min-height: 0;
    }

    .graph {
        padding-top: 16px;
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
        padding: 6px 4px 6px 0;
        min-height: 0;
        overscroll-behavior: contain;
        will-change: scroll-position;
    }

    .list-card::-webkit-scrollbar {
        width: 8px;
    }
    .list-card::-webkit-scrollbar-track {
        background: transparent;
        margin-top: 4px;
        margin-bottom: 4px;
    }
    .list-card::-webkit-scrollbar-thumb {
        background: var(--color-border-strong);
        border-radius: 999px;
        background-clip: padding-box;
        border: 2px solid transparent;
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
