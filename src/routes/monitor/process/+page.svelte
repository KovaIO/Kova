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

    import type { FlatProcess, Metrics, ProcessNode } from "../types";
    import { metricLabel } from "../utils/format";
    import { getProcessHierarchy } from "../utils/process-tree";

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

    let cpuValue = 0;
    let ramValue = 0;
    let diskValue = 0;
    let networkBps = 0;

    let expandedPids = new Set<number>();

    function toggleExpand(id: number) {
        const next = new Set(expandedPids);
        next.has(id) ? next.delete(id) : next.add(id);
        expandedPids = next;
    }

    function applyMetrics(m: Metrics) {
        cpuValue = m.cpu_percent;
        ramValue = m.ram_percent;
        diskValue = m.disk_percent;
        networkBps = m.network_bps;

        const hierarchy = getProcessHierarchy(m.processes, pid);

        if (!hierarchy.target) return;

        proc = hierarchy.target;
        ancestors = hierarchy.ancestors;
        rootNode = hierarchy.rootNode;

        cpuHistory = [...cpuHistory.slice(-59), proc.cpu_percent];

        ramHistory = [...ramHistory.slice(-59), proc.ram_bytes];

        networkHistory = [...networkHistory.slice(-59), proc.net_bps];
    }

    async function openMonitor() {
        await invoke("open_monitor", { tab: activeTab });
    }

    let unlisten: UnlistenFn;

    onMount(async () => {
        const snap = await invoke<Metrics | null>("get_current_metrics");
        if (snap) applyMetrics(snap);
        unlisten = await listen<Metrics>("metrics", (e) =>
            applyMetrics(e.payload),
        );
    });

    onDestroy(() => unlisten?.());
</script>

<div class="page">
    <button type="button" class="back-button" onclick={openMonitor}>
        <ArrowLeft size={14} />
        <span>Monitor</span>
    </button>

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

    <div class="card details-card">
        {#if proc}
            <div class="main-row">
                {@render procIcon(proc)}
                <div class="proc-info">
                    <div class="proc-name">{proc.name}</div>
                    <div class="proc-sub">PID {proc.pid}</div>
                </div>
                <div class="proc-metric-main">
                    {metricLabel(proc, activeTab)}
                </div>
            </div>

            <div class="divider"></div>

            <div class="meta-block">
                <div class="meta-label">Started at</div>
                <div class="meta-value">
                    {new Date(proc.started_at * 1000).toLocaleString()}
                </div>
            </div>

            {#if proc.exe_path}
                <div class="meta-block">
                    <div class="meta-label">Path</div>
                    <div class="meta-value path">{proc.exe_path}</div>
                </div>
            {/if}

            <div class="meta-block">
                <div class="meta-label">Process hierarchy</div>
            </div>

            <div class="hierarchy-section">
                {#each ancestors.slice(0, -1) as anc, i (anc.pid)}
                    <div class="anc-row" style:padding-left="{i * 14 + 16}px">
                        <span class="connector" aria-hidden="true"
                            >{i > 0 ? "╰" : ""}</span
                        >
                        {@render procIcon(anc)}
                        <span class="anc-name">{anc.name}</span>
                        <span class="anc-pid">{anc.pid}</span>
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
                {#if open}<ChevronDown size={11} />{:else}<ChevronRight
                        size={11}
                    />{/if}
            {:else}
                <span class="chevron-spacer"></span>
            {/if}
        </span>

        {@render procIcon(node)}

        <span class="proc-name" class:is-target={isTarget}>{node.name}</span>

        {#if hasKids}
            <span class="child-badge">{node.children.length}</span>
        {/if}

        <span class="proc-metric" class:is-target={isTarget}>
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

    .back-button {
        display: flex;
        align-items: center;
        gap: 6px;
        align-self: flex-start;
        padding: 4px 10px 4px 8px;
        border: none;
        background: rgba(255, 255, 255, 0.07);
        border-radius: var(--radius-sm, 6px);
        color: var(--color-text-dim);
        font: inherit;
        font-size: 12px;
        cursor: pointer;
        transition:
            background 0.12s,
            color 0.12s;
    }

    .back-button:hover {
        background: rgba(255, 255, 255, 0.12);
        color: var(--color-text-secondary);
    }

    .back-button:focus {
        outline: none;
    }
    .back-button:focus-visible {
        background: rgba(255, 255, 255, 0.12);
        color: var(--color-text-secondary);
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

    .details-card {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow-y: auto;
        padding: 4px 0 12px;
        contain: strict;
        overscroll-behavior: contain;
    }

    .details-card::-webkit-scrollbar {
        width: 3px;
    }
    .details-card::-webkit-scrollbar-track {
        background: transparent;
    }
    .details-card::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 999px;
    }

    .main-row {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 16px 10px;
    }

    .divider {
        height: 1px;
        background: var(--color-border-subtle);
        margin: 0 16px 4px;
    }

    .proc-info {
        flex: 1;
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

    .proc-sub {
        margin-top: 2px;
        font-size: 11px;
        color: var(--color-text-dim);
    }

    .proc-metric-main {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-secondary);
        letter-spacing: 0.02em;
        flex-shrink: 0;
        min-width: 60px;
        text-align: right;
        font-variant-numeric: tabular-nums;
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

    .fallback {
        background: rgba(255, 255, 255, 0.08);
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

    .meta-block {
        display: flex;
        flex-direction: column;
        padding: 10px 16px 4px;
        gap: 6px;
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
        line-height: 1.4;
    }

    .path {
        word-break: break-all;
    }

    .hierarchy-section {
        display: flex;
        flex-direction: column;
        padding-top: 4px;
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
        font-size: 11px;
        color: var(--color-text-dim);
        opacity: 0.4;
        line-height: 1;
        flex-shrink: 0;
        width: 10px;
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
        font-size: 10.5px;
        color: var(--color-text-dim);
        font-variant-numeric: tabular-nums;
        flex-shrink: 0;
        opacity: 0.5;
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
        cursor: default;
        transition: background 0.12s;
    }

    .process-row:hover {
        background: var(--color-button-bg-hover);
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
        justify-content: center;
        color: var(--color-text-dim);
        cursor: pointer;
    }

    .chevron-spacer {
        display: inline-block;
        width: 11px;
        height: 11px;
    }

    .proc-name {
        flex: 1;
        font-size: 13px;
        font-weight: 450;
        color: var(--color-text-dim);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .proc-name.is-target {
        color: var(--color-text-secondary);
        font-weight: 500;
    }

    .child-badge {
        font-size: 11px;
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

    .proc-metric.is-target {
        color: var(--color-text-secondary);
    }

    .empty {
        text-align: center;
        color: var(--color-text-dim);
        font-size: 12px;
        padding: 24px 0;
    }
</style>
