<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { page } from "$app/state";
    import {
        ChevronRight,
        ChevronDown,
        Cpu,
        Terminal,
        ArrowLeft,
    } from "@lucide/svelte";
    import MetricGraph from "$components/MetricGraph.svelte";

    import type { FlatProcess, Metrics, ProcessNode } from "$types/metrics";
    import { metricLabel } from "$utils/format";
    import { getProcessHierarchy } from "$utils/process-tree";

    const pid: number = Number(page.url.searchParams.get("pid"));

    let activeTab: "cpu" | "ram" | "network" = (() => {
        const t = page.url.searchParams.get("tab");
        return t === "ram" || t === "network" ? t : "cpu";
    })();

    let proc: FlatProcess | null = null;
    let ancestors: FlatProcess[] = [];
    let rootNode: ProcessNode | null = null;

    let cpuHistory: number[] = [];
    let ramHistory: number[] = [];
    let networkHistory: number[] = [];

    let ramMax = 1;
    let netMax = 1;

    let expandedPids = new Set<number>();

    let selectedHistoryIndex: number | null = null;
    let historyMode = false;

    async function handleBarClick(index: number) {
        if (selectedHistoryIndex === index) {
            exitHistoryMode();
            return;
        }

        historyMode = true;
        selectedHistoryIndex = index;

        const snap = await invoke<Metrics>("get_snapshot", { index });

        applyMetrics(snap, false);
    }

    function exitHistoryMode() {
        historyMode = false;
        selectedHistoryIndex = null;
    }

    function toggleExpand(id: number) {
        const next = new Set(expandedPids);
        next.has(id) ? next.delete(id) : next.add(id);
        expandedPids = next;
    }

    function applyMetrics(m: Metrics, appendHistory = true) {
        const hierarchy = getProcessHierarchy(m.processes, pid);
        if (!hierarchy.target) return;

        proc = hierarchy.target;
        ancestors = hierarchy.ancestors;
        rootNode = hierarchy.rootNode;

        if (appendHistory) {
            cpuHistory = [...cpuHistory.slice(-59), proc.cpu_percent];
            ramHistory = [...ramHistory.slice(-59), proc.ram_bytes];
            networkHistory = [...networkHistory.slice(-59), proc.net_bps];
        }

        ramMax = Math.max(...ramHistory, 1);
        netMax = Math.max(...networkHistory, 1);
    }

    async function openMonitor() {
        await invoke("open_monitor", { tab: activeTab });
    }

    let unlisten: UnlistenFn;
    onMount(async () => {
        const [cpuH, ramH, netH] = await invoke<[number[], number[], number[]]>(
            "get_process_history",
            { pid },
        );
        cpuHistory = cpuH;
        ramHistory = ramH;
        networkHistory = netH;
        ramMax = Math.max(...ramH, 1);
        netMax = Math.max(...netH, 1);

        const snap = await invoke<Metrics | null>("get_current_metrics");
        if (snap) applyMetrics(snap, false);

        unlisten = await listen<Metrics>("metrics", async (e) => {
            if (!historyMode) {
                applyMetrics(e.payload, true);
                return;
            }

            if (selectedHistoryIndex !== null) {
                selectedHistoryIndex -= 1;

                const snap = await invoke<Metrics>("get_snapshot", {
                    index: selectedHistoryIndex,
                });

                applyMetrics(snap, true);
            }
        });
    });

    async function quitProcess(force = false) {
        try {
            if (force) {
                await invoke("force_quit_process_cmd", { pid });
            } else {
                await invoke("quit_process_cmd", { pid });
            }

            await openMonitor();
        } catch (err) {
            console.error("Failed to quit process:", err);
        }
    }

    onDestroy(() => unlisten?.());
</script>

<div class="page">
    <div class="card unified-card">
        <div class="topbar">
            <button type="button" class="back-button" onclick={openMonitor}>
                <ArrowLeft size={14} />
            </button>
        </div>

        <MetricGraph
            bind:activeTab
            {cpuHistory}
            {ramHistory}
            networkHistory={networkHistory.map((v) => (v / netMax) * 100)}
            {ramMax}
            {netMax}
            diskCleanEnabled={false}
            onBarClick={handleBarClick}
            selectedIndex={selectedHistoryIndex}
            onBackgroundClick={exitHistoryMode}
        />

        <div class="details-scroll">
            {#if proc}
                <div class="main-row">
                    {@render procIcon(proc)}

                    <div class="proc-name">{proc.name}</div>
                </div>

                <div class="meta-block">
                    <div class="meta-label">Started at</div>
                    <div class="meta-value">
                        {new Date(proc.started_at * 1000).toLocaleString()}
                    </div>
                </div>

                {#if proc.exe_path}
                    <div class="meta-block">
                        <div class="meta-label">Path</div>
                        <div class="meta-value path">
                            {proc.exe_path}
                        </div>
                    </div>
                {/if}

                <div class="meta-block">
                    <div class="meta-label">Process hierarchy</div>
                </div>

                <div class="hierarchy-section">
                    {#each ancestors.slice(0, -1) as anc, i (anc.pid)}
                        <div
                            class="anc-row"
                            style:padding-left="{i * 14 + 16}px"
                        >
                            <span class="connector">
                                {i > 0 ? "╰" : ""}
                            </span>

                            {@render procIcon(anc)}

                            <span class="anc-name">
                                {anc.name}
                            </span>

                            <span class="anc-pid">
                                {anc.pid}
                            </span>
                        </div>
                    {/each}

                    {#if rootNode}
                        {@render treeRow(rootNode, (ancestors.length - 1) * 14)}
                    {/if}
                </div>
            {:else}
                <p class="empty">Waiting for process data…</p>
            {/if}
        </div>

        <div class="actions">
            <button
                type="button"
                class="action-button danger"
                onclick={() => quitProcess(false)}
            >
                Force quit
            </button>
            <button
                type="button"
                class="action-button"
                onclick={() => quitProcess(true)}
            >
                Quit
            </button>
        </div>
    </div>
</div>

{#snippet procIcon(p: { icon?: string | null })}
    {#if p.icon === "system"}
        <div class="proc-icon fallback-icon"><Cpu size={13} /></div>
    {:else if p.icon === "terminal"}
        <div class="proc-icon fallback-icon"><Terminal size={13} /></div>
    {:else if p.icon}
        <img class="proc-icon" src={`data:image/png;base64,${p.icon}`} alt="" />
    {:else}
        <div class="proc-icon fallback"></div>
    {/if}
{/snippet}

{#snippet treeRow(node: ProcessNode, baseIndent: number)}
    {@const hasKids = node.children.length > 0}
    {@const open = expandedPids.has(node.pid)}
    {@const isTarget = node.pid === pid}

    <button
        type="button"
        class="process-row"
        style:padding-left="{baseIndent + 16}px"
        onclick={() => {
            if (hasKids) toggleExpand(node.pid);
        }}
    >
        <span
            class="chevron"
            aria-hidden="true"
            onclick={(e) => {
                e.stopPropagation();
                if (hasKids) toggleExpand(node.pid);
            }}
        >
            {#if hasKids}
                {#if open}<ChevronDown size={14} strokeWidth={3} />
                {:else}<ChevronRight size={14} strokeWidth={3} />
                {/if}
            {:else}
                <span class="chevron-spacer"></span>
            {/if}
        </span>

        {@render procIcon(node)}

        <div class="proc-main-info">
            <span class="proc-name">
                {node.name}
            </span>

            {#if hasKids}
                <span class="child-badge">
                    <span class="plus">+</span>
                    <span>{node.children.length}</span>
                </span>
            {/if}
        </div>

        <span class="proc-metric">
            {metricLabel(node, activeTab)}
        </span>
    </button>

    {#if hasKids && open}
        {#each node.children as child (`${child.pid}-${activeTab}`)}
            {@render treeRow(child, baseIndent + 16)}
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

    .unified-card {
        height: 100%;
        display: flex;
        flex-direction: column;
    }

    .topbar {
        display: flex;
        align-items: center;
        padding: 12px 12px 0;
        flex-shrink: 0;
    }

    .back-button {
        display: flex;
        align-items: center;

        padding: 8px;

        border: none;
        border-radius: var(--radius-md);

        background: var(--color-main-bg);

        color: var(--color-text-secondary);

        font: inherit;
        font-size: 14px;

        transition: var(--transition-fast);
    }

    .back-button:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-secondary);
    }

    .back-button:focus {
        outline: none;
    }

    .back-button:focus-visible {
        background: var(--color-button-bg-hover);
        color: var(--color-text-secondary);
    }

    .back-button:active {
        transform: scale(0.98);
    }

    .details-scroll {
        flex: 1;
        min-height: 0;

        overflow-y: auto;
        overflow-x: hidden;

        padding: 10px 4px 12px 0;

        contain: strict;
        overscroll-behavior: contain;
    }

    .details-scroll::-webkit-scrollbar {
        width: 8px;
    }

    .details-scroll::-webkit-scrollbar-track {
        background: transparent;
        margin-top: 4px;
        margin-bottom: 4px;
    }

    .details-scroll::-webkit-scrollbar-thumb {
        background: var(--color-border-strong);
        border-radius: 999px;
        background-clip: padding-box;
        border: 2px solid transparent;
    }

    .main-row {
        display: flex;
        align-items: center;
        gap: 14px;

        padding: 14px 16px 12px;
    }

    .proc-main-info {
        flex: 1;

        display: flex;
        align-items: center;
        gap: 8px;

        min-width: 0;
    }

    .proc-name {
        font-size: 13px;
        font-weight: 500;

        color: var(--color-text-secondary);

        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .main-row .proc-name {
        font-size: 18px;
        font-weight: 600;
        color: var(--color-text-secondary);
        letter-spacing: -0.01em;
    }

    .proc-icon {
        width: 20px;
        height: 20px;

        flex-shrink: 0;

        border-radius: 4px;

        object-fit: contain;
    }

    .main-row .proc-icon {
        width: 26px;
        height: 26px;
        border-radius: 8px;
    }

    img.proc-icon {
        object-fit: contain;
    }

    .fallback {
        background: var(--color-track-fill);
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

    .main-row .fallback-icon :global(svg) {
        width: 18px;
        height: 18px;
    }

    .meta-block {
        display: flex;
        flex-direction: column;
        gap: 6px;

        padding: 12px 16px 4px;
    }

    .meta-label {
        font-size: 11px;

        color: var(--color-text-dim);

        letter-spacing: 0.03em;

        text-transform: uppercase;
    }

    .meta-value {
        font-size: 12.5px;

        color: var(--color-text-secondary);

        line-height: 1.45;
    }

    .path {
        word-break: break-all;
    }

    .hierarchy-section {
        display: flex;
        flex-direction: column;

        padding-top: 8px;
    }

    .anc-row {
        display: flex;
        align-items: center;
        gap: 6px;

        min-height: 32px;

        padding-right: 16px;

        box-sizing: border-box;
    }

    .connector {
        width: 10px;

        flex-shrink: 0;

        font-size: 11px;
        line-height: 1;

        color: var(--color-text-dim);

        opacity: 0.4;
    }

    .anc-name {
        flex: 1;

        font-size: 12.5px;
        font-weight: 450;

        color: var(--color-text-dim);

        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .anc-pid {
        display: none;
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
    }

    .chevron {
        width: 22px;

        flex-shrink: 0;

        display: flex;
        align-items: center;
        justify-content: center;

        color: var(--color-text-tertiary);
    }

    .chevron-spacer {
        display: inline-block;
        width: 11px;
        height: 11px;
    }

    .child-badge {
        display: inline-flex;
        align-items: center;
        gap: 3px;

        flex-shrink: 0;
        padding: 1px 6px;
        border-radius: var(--radius-md);
        background: var(--color-track-fill);
        font-size: 12px;
        color: var(--color-text-secondary);
    }

    .plus {
        position: relative;
        top: -0.5px;

        font-size: 11px;
        font-weight: 600;
    }

    .proc-metric {
        min-width: 60px;

        flex-shrink: 0;

        text-align: right;

        font-size: 13px;
        font-weight: 500;

        letter-spacing: 0.02em;

        color: var(--color-text-secondary);

        font-variant-numeric: tabular-nums;
    }

    .empty {
        padding: 24px 0;

        text-align: center;

        font-size: 12px;

        color: var(--color-text-dim);
    }

    .actions {
        display: flex;
        gap: 8px;
        padding: 12px 16px 16px;
        width: 100%;
        box-sizing: border-box;
        flex-shrink: 0;
    }

    .action-button {
        flex: 1;
        height: 36px;
        border: none;
        border-radius: var(--radius-md);
        background: var(--color-accent);
        color: var(--color-text-primary);
        font: inherit;
        font-size: 12px;
        font-weight: 500;
        transition: var(--transition-medium);
    }
    .action-button:hover {
        background: var(--color-accent-hover);
    }

    .action-button.danger {
        background: var(--color-track-fill);
    }
    .action-button.danger:hover {
        background: var(--color-danger);
    }
</style>
