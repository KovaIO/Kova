<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import {
        ArrowLeft,
        ChevronDown,
        ChevronRight,
        Folder,
        HardDrive,
        Search,
        Trash2,
        AlertTriangle,
        ShieldCheck,
        ShieldAlert,
        Code,
        Globe,
        Layers,
        LayoutGrid,
        Settings,
    } from "@lucide/svelte";
    import {
        deleteAllDiskItems,
        deleteDiskItems,
        fetchDiskItemDetail,
        fetchDiskScanPreview,
        fetchDiskScanResult,
        fetchDiskVolumeInfo,
        startDiskScan,
    } from "$services/disk";
    import type {
        DiskCategory,
        DiskItemDetail,
        DiskScanItem,
        DiskScanProgress,
        DiskScanResult,
        DiskSafety,
        DiskVolumeInfo,
        ScanPreview,
    } from "$types/disk";
    import { formatLastScan, safetyLabel } from "$types/disk";
    import { formatBytes } from "$utils/format";

    type View = "ready" | "scanning" | "results" | "detail";

    let view: View = $state("ready");
    let volume: DiskVolumeInfo | null = $state(null);
    let preview: ScanPreview | null = $state(null);
    let result: DiskScanResult | null = $state(null);
    let detail: DiskItemDetail | null = $state(null);
    let scanProgress = $state(0);
    let scanMessage = $state("");
    let expandedCategories = $state(new Set<DiskCategory>());
    let confirmDeleteAll = $state(false);
    let confirmDeleteId: string | null = $state(null);
    let loading = $state(true);
    let unlisten: UnlistenFn | undefined;

    let reclaimable = $derived((result as DiskScanResult | null)?.total_reclaimable ?? 0);

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

    async function loadInitial() {
        loading = true;
        try {
            [volume, preview, result] = await Promise.all([
                fetchDiskVolumeInfo(),
                fetchDiskScanPreview(),
                fetchDiskScanResult(),
            ]);
            view = result ? "results" : "ready";
        } finally {
            loading = false;
        }
    }

    async function handleScan() {
        view = "scanning";
        scanProgress = 0;
        scanMessage = "Starting scan…";
        try {
            result = await startDiskScan();
            volume = await fetchDiskVolumeInfo();
            view = "results";
            expandedCategories = new Set(
                result.categories.map((c) => c.category),
            );
        } catch {
            view = result ? "results" : "ready";
        }
    }

    async function openDetail(item: DiskScanItem) {
        detail = await fetchDiskItemDetail(item.id);
        view = "detail";
    }

    function toggleCategory(category: DiskCategory) {
        const next = new Set(expandedCategories);
        if (next.has(category)) next.delete(category);
        else next.add(category);
        expandedCategories = next;
    }

    async function deleteOne(id: string) {
        await deleteDiskItems([id]);
        confirmDeleteId = null;
        result = await fetchDiskScanResult();
        volume = await fetchDiskVolumeInfo();
        if (detail?.item.id === id) {
            detail = null;
            view = result?.categories.length ? "results" : "ready";
        }
        if (!result?.categories.length) view = "ready";
    }

    async function deleteAll() {
        await deleteAllDiskItems();
        confirmDeleteAll = false;
        result = await fetchDiskScanResult();
        volume = await fetchDiskVolumeInfo();
        view = "ready";
    }

    function categoryIcon(category: DiskCategory) {
        switch (category) {
            case "system":
                return Settings;
            case "browsers":
                return Globe;
            case "development":
                return Code;
            case "applications":
                return LayoutGrid;
            case "storage":
                return Folder;
            default:
                return Layers;
        }
    }

    function safetyIcon(safety: DiskSafety) {
        switch (safety) {
            case "safe":
                return ShieldCheck;
            case "caution":
                return ShieldAlert;
            default:
                return AlertTriangle;
        }
    }

    onMount(() => {
        void loadInitial();
        (async () => {
            unlisten = await listen<DiskScanProgress>(
                "disk-scan-progress",
                (e) => {
                    scanProgress = e.payload.progress;
                    scanMessage = e.payload.message;
                },
            );
        })();
    });

    onDestroy(() => unlisten?.());
</script>

<div class="disk-panel">
    {#if loading}
        <p class="empty">Loading disk info…</p>
    {:else if view === "detail" && detail}
        <div class="detail-view">
            <button
                type="button"
                class="back-btn"
                onclick={() => (view = "results")}
            >
                <ArrowLeft size={14} />
                Back
            </button>

            <div class="card detail-card">
                <div class="detail-head">
                    <div class="detail-icon">
                        <Folder size={18} />
                    </div>
                    <div class="detail-titles">
                        <h2>{detail.item.name}</h2>
                        <p class="detail-path">{detail.item.path}</p>
                    </div>
                </div>

                <div class="meta-grid">
                    <div class="meta-row">
                        <span>Size</span>
                        <span>{formatBytes(detail.item.size_bytes)}</span>
                    </div>
                    <div class="meta-row">
                        <span>Items</span>
                        <span>{detail.item.item_count}</span>
                    </div>
                    <div class="meta-row">
                        <span>Category</span>
                        <span class="capitalize">{detail.item.category}</span>
                    </div>
                    <div class="meta-row">
                        <span>Type</span>
                        <span>{detail.item.is_dir ? "Folder" : "File"}</span>
                    </div>
                </div>

                <div
                    class="safety-banner"
                    class:safe={detail.item.safety === "safe"}
                    class:caution={detail.item.safety === "caution"}
                    class:unsafe={detail.item.safety === "unsafe"}
                >
                    {#if safetyIcon(detail.item.safety)}
                        {@const Icon = safetyIcon(detail.item.safety)}
                        <Icon size={15} />
                    {/if}
                    <div>
                        <strong>{safetyLabel(detail.item.safety)}</strong>
                        <p>{detail.item.safety_reason}</p>
                    </div>
                </div>

                {#if detail.children.length > 0}
                    <div class="children-section">
                        <span class="section-label">Contents</span>
                        <div class="children-list">
                            {#each detail.children as child}
                                <div class="child-row">
                                    <Folder size={13} />
                                    <span class="child-name">{child.name}</span>
                                    <span class="child-size"
                                        >{formatBytes(child.size_bytes)}</span
                                    >
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}

                {#if detail.item.safety !== "unsafe"}
                    <div class="detail-actions">
                        {#if confirmDeleteId === detail.item.id}
                            <button
                                type="button"
                                class="btn danger"
                                onclick={() => void deleteOne(detail!.item.id)}
                            >
                                Confirm delete
                            </button>
                            <button
                                type="button"
                                class="btn ghost"
                                onclick={() => (confirmDeleteId = null)}
                            >
                                Cancel
                            </button>
                        {:else}
                            <button
                                type="button"
                                class="btn danger"
                                onclick={() =>
                                    (confirmDeleteId = detail!.item.id)}
                            >
                                <Trash2 size={14} />
                                Delete
                            </button>
                        {/if}
                    </div>
                {/if}
            </div>
        </div>
    {:else}
        <div class="card summary-card">
            <div class="summary-top">
                <div class="disk-square">
                    <div
                        class="disk-square-fill"
                        style="height: {volume?.used_percent ?? 0}%; background: linear-gradient(to top, {diskColor(volume?.used_percent ?? 0)}, {diskColorHover(volume?.used_percent ?? 0)});"
                    ></div>
                    <div class="disk-square-content">
                        <HardDrive size={16} />
                        <span>{volume?.used_percent ?? 0}%</span>
                    </div>
                </div>
                <div class="summary-stats">
                    <div class="stat-row">
                        <span>Total</span>
                        <span>{formatBytes(volume?.total_bytes ?? 0)}</span>
                    </div>
                    <div class="stat-row">
                        <span>Used</span>
                        <span>{formatBytes(volume?.used_bytes ?? 0)}</span>
                    </div>
                    <div class="stat-row">
                        <span>Free</span>
                        <span>{formatBytes(volume?.available_bytes ?? 0)}</span>
                    </div>
                    <div class="stat-row muted">
                        <span>{volume?.label ?? "Disk"}</span>
                        <span>Last scan: {formatLastScan(volume?.last_scan_at)}</span>
                    </div>
                </div>
            </div>
        </div>

        {#if view === "scanning"}
            <div class="card scan-card">
                <div class="scan-animation">
                    <div class="scan-ring"></div>
                    <Search size={18} class="scan-icon" />
                </div>
                <p class="scan-message">{scanMessage}</p>
                <div class="progress-track">
                    <div
                        class="progress-fill"
                        style="width: {scanProgress}%"
                    ></div>
                </div>
            </div>
        {:else if view === "ready"}
            <div class="card ready-card">
                <div class="ready-head">
                    <HardDrive size={18} />
                    <div>
                        <h3>Ready to clean</h3>
                        <p>
                            Scan {volume?.mount_path ?? "your disk"} for
                            reclaimable cache and temporary files.
                        </p>
                    </div>
                </div>

                <div class="preview-list">
                    <span class="section-label">What will be scanned</span>
                    {#each preview?.categories ?? [] as cat}
                        <div class="preview-row">
                            {#if categoryIcon(cat.category)}
                                {@const Icon = categoryIcon(cat.category)}
                                <Icon size={14} />
                            {/if}
                            <span class="preview-name">{cat.label}</span>
                            <span class="preview-count"
                                >{cat.target_count} targets</span
                            >
                        </div>
                    {/each}
                </div>

                <button
                    type="button"
                    class="btn primary scan-btn"
                    onclick={() => void handleScan()}
                >
                    <Search size={14} />
                    Scan now
                </button>
            </div>
        {:else if view === "results" && result}
            <div class="card results-card">
                <div class="results-banner">
                    <ShieldCheck size={14} />
                    <span
                        >Scan complete · {formatBytes(reclaimable)} reclaimable</span
                    >
                </div>

                <div class="categories">
                    {#each result.categories as group}
                        {@const open = expandedCategories.has(group.category)}
                        <div class="category-block">
                            <button
                                type="button"
                                class="category-head"
                                onclick={() => toggleCategory(group.category)}
                            >
                                {#if open}
                                    <ChevronDown size={14} />
                                {:else}
                                    <ChevronRight size={14} />
                                {/if}
                                {#if categoryIcon(group.category)}
                                    {@const Icon = categoryIcon(group.category)}
                                    <Icon size={14} />
                                {/if}
                                <span class="category-name">{group.label}</span>
                                <span class="category-size"
                                    >{formatBytes(group.total_bytes)}</span
                                >
                            </button>

                            {#if open}
                                <div class="items-list">
                                    {#each group.items as item}
                                        <div class="item-row">
                                            <button
                                                type="button"
                                                class="item-main"
                                                onclick={() =>
                                                    void openDetail(item)}
                                            >
                                                <Folder size={13} />
                                                <span class="item-name"
                                                    >{item.name}</span
                                                >
                                                <span class="item-size"
                                                    >{formatBytes(
                                                        item.size_bytes,
                                                    )}</span
                                                >
                                            </button>
                                            {#if item.safety !== "unsafe"}
                                                {#if confirmDeleteId === item.id}
                                                    <button
                                                        type="button"
                                                        class="icon-btn confirm"
                                                        onclick={() =>
                                                            void deleteOne(
                                                                item.id,
                                                            )}
                                                    >
                                                        OK
                                                    </button>
                                                {:else}
                                                    <button
                                                        type="button"
                                                        class="icon-btn"
                                                        title="Delete"
                                                        onclick={() =>
                                                            (confirmDeleteId =
                                                                item.id)}
                                                    >
                                                        <Trash2 size={13} />
                                                    </button>
                                                {/if}
                                            {/if}
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>

                <div class="results-footer">
                    {#if confirmDeleteAll}
                        <p class="warning-text">
                            Delete all safe and reviewable items? This cannot be
                            undone.
                        </p>
                        <div class="footer-actions">
                            <button
                                type="button"
                                class="btn danger"
                                onclick={() => void deleteAll()}
                            >
                                Confirm delete all
                            </button>
                            <button
                                type="button"
                                class="btn ghost"
                                onclick={() => (confirmDeleteAll = false)}
                            >
                                Cancel
                            </button>
                        </div>
                    {:else}
                        <button
                            type="button"
                            class="btn ghost"
                            onclick={() => void handleScan()}
                        >
                            Scan again
                        </button>
                        <button
                            type="button"
                            class="btn danger"
                            onclick={() => (confirmDeleteAll = true)}
                        >
                            <Trash2 size={14} />
                            Delete all ({formatBytes(reclaimable)})
                        </button>
                    {/if}
                </div>
            </div>
        {/if}
    {/if}
</div>

<style>
    .disk-panel {
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex: 1;
        min-height: 0;
        overflow: hidden;
    }

    .card {
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        color: var(--color-text-primary);
    }

    .summary-card {
        padding: 14px;
        flex-shrink: 0;
    }

    .summary-top {
        display: flex;
        gap: 14px;
        align-items: center;
    }

    .disk-square {
        position: relative;
        width: 64px;
        height: 64px;
        border-radius: var(--radius-sm);
        background: var(--color-track-fill);
        display: grid;
        place-items: center;
        flex-shrink: 0;
        overflow: hidden;
    }

    .disk-square-fill {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        transition: height 600ms cubic-bezier(0.4, 0, 0.2, 1);
        border-radius: 0 0 var(--radius-sm) var(--radius-sm);
        opacity: 0.9;
    }

    .disk-square-content {
        position: relative;
        z-index: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 2px;
        color: var(--color-text-secondary);
        font-size: 13px;
        font-weight: 600;
    }

    .summary-stats {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .stat-row {
        display: flex;
        justify-content: space-between;
        font-size: 12px;
        color: var(--color-text-secondary);
    }

    .stat-row span:last-child {
        font-variant-numeric: tabular-nums;
        color: var(--color-text-primary);
    }

    .stat-row.muted {
        margin-top: 4px;
        font-size: 11px;
        color: var(--color-text-dim);
    }

    .ready-card,
    .scan-card,
    .results-card,
    .detail-card {
        padding: 14px;
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .ready-head {
        display: flex;
        gap: 10px;
        margin-bottom: 12px;
        color: var(--color-text-secondary);
    }

    .ready-head h3 {
        font-size: 14px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin-bottom: 4px;
    }

    .ready-head p {
        font-size: 12px;
        line-height: 1.45;
        color: var(--color-text-muted);
    }

    .section-label {
        display: block;
        font-size: 11px;
        font-weight: 500;
        color: var(--color-text-dim);
        text-transform: uppercase;
        letter-spacing: 0.04em;
        margin-bottom: 8px;
    }

    .preview-list {
        flex: 1;
        overflow-y: auto;
        margin-bottom: 12px;
        scrollbar-width: none;
    }

    .preview-list::-webkit-scrollbar {
        display: none;
    }

    .preview-row {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 7px 0;
        border-bottom: 1px solid var(--color-border-subtle);
        font-size: 12.5px;
        color: var(--color-text-secondary);
    }

    .preview-name {
        flex: 1;
    }

    .preview-count {
        font-size: 11px;
        color: var(--color-text-dim);
    }

    .scan-card {
        align-items: center;
        justify-content: center;
        gap: 12px;
    }

    .scan-animation {
        position: relative;
        width: 48px;
        height: 48px;
        display: grid;
        place-items: center;
        color: var(--color-accent);
    }

    .scan-ring {
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 2px solid var(--color-accent-border);
        border-top-color: var(--color-accent);
        animation: spin 900ms linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .scan-message {
        font-size: 12.5px;
        color: var(--color-text-secondary);
    }

    .progress-track {
        width: 100%;
        max-width: 220px;
        height: 4px;
        background: var(--color-track-fill);
        border-radius: 999px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: var(--color-accent);
        transition: width 200ms ease;
    }

    .results-banner {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 10px;
        margin-bottom: 10px;
        border-radius: var(--radius-sm);
        background: var(--color-success-soft);
        color: var(--color-success);
        font-size: 12px;
        flex-shrink: 0;
    }

    .categories {
        flex: 1;
        overflow-y: auto;
        scrollbar-width: none;
    }

    .categories::-webkit-scrollbar {
        display: none;
    }

    .category-block {
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .category-head {
        width: 100%;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 4px;
        border: none;
        background: transparent;
        color: var(--color-text-secondary);
        font: inherit;
        font-size: 13px;
        cursor: pointer;
        text-align: left;
    }

    .category-head:hover {
        color: var(--color-text-primary);
    }

    .category-name {
        flex: 1;
        font-weight: 500;
    }

    .category-size {
        font-variant-numeric: tabular-nums;
        font-size: 12px;
    }

    .items-list {
        padding: 0 4px 8px 22px;
    }

    .item-row {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .item-main {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 7px 4px;
        border: none;
        background: transparent;
        color: inherit;
        font: inherit;
        font-size: 12.5px;
        text-align: left;
        cursor: pointer;
        border-radius: var(--radius-sm);
    }

    .item-main:hover {
        background: var(--color-button-bg);
    }

    .item-name {
        flex: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        color: var(--color-text-secondary);
    }

    .item-size {
        font-size: 12px;
        color: var(--color-text-muted);
        font-variant-numeric: tabular-nums;
    }

    .icon-btn {
        width: 28px;
        height: 28px;
        border: none;
        border-radius: 6px;
        background: transparent;
        color: var(--color-text-dim);
        display: grid;
        place-items: center;
        cursor: pointer;
        opacity: 0;
        transition: opacity var(--transition-fast);
    }

    .item-row:hover .icon-btn {
        opacity: 1;
    }

    .icon-btn:hover {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    .icon-btn.confirm {
        opacity: 1;
        font-size: 11px;
        font-weight: 600;
        color: var(--color-danger);
    }

    .results-footer {
        display: flex;
        flex-direction: column;
        gap: 8px;
        padding-top: 10px;
        border-top: 1px solid var(--color-border-subtle);
        flex-shrink: 0;
    }

    .footer-actions {
        display: flex;
        gap: 8px;
    }

    .warning-text {
        font-size: 12px;
        color: var(--color-warning);
        line-height: 1.4;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 7px;
        padding: 9px 14px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        cursor: pointer;
        transition: all var(--transition-fast);
    }

    .btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .btn.primary {
        background: var(--color-accent-soft);
        border-color: var(--color-accent-border);
        color: var(--color-accent);
    }

    .btn.primary:hover {
        background: var(--color-accent);
        color: white;
    }

    .btn.danger {
        border-color: var(--color-danger-soft);
        color: var(--color-danger);
    }

    .btn.danger:hover {
        background: var(--color-danger-soft);
    }

    .btn.ghost {
        background: transparent;
    }

    .scan-btn {
        width: 100%;
    }

    .detail-view {
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex: 1;
        min-height: 0;
    }

    .back-btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        align-self: flex-start;
        padding: 6px 10px;
        border: none;
        background: transparent;
        color: var(--color-text-muted);
        font-size: 12px;
        font-family: inherit;
        cursor: pointer;
        border-radius: var(--radius-sm);
    }

    .back-btn:hover {
        background: var(--color-button-bg);
        color: var(--color-text-primary);
    }

    .detail-head {
        display: flex;
        gap: 10px;
        margin-bottom: 12px;
    }

    .detail-icon {
        width: 36px;
        height: 36px;
        border-radius: 8px;
        background: var(--color-track-fill);
        display: grid;
        place-items: center;
        color: var(--color-text-secondary);
    }

    .detail-titles h2 {
        font-size: 14px;
        font-weight: 600;
        margin-bottom: 4px;
    }

    .detail-path {
        font-size: 11px;
        color: var(--color-text-dim);
        word-break: break-all;
    }

    .meta-grid {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 12px;
    }

    .meta-row {
        display: flex;
        justify-content: space-between;
        font-size: 12px;
        color: var(--color-text-muted);
    }

    .meta-row span:last-child {
        color: var(--color-text-secondary);
    }

    .capitalize {
        text-transform: capitalize;
    }

    .safety-banner {
        display: flex;
        gap: 10px;
        padding: 10px 12px;
        border-radius: var(--radius-sm);
        margin-bottom: 12px;
        font-size: 12px;
        line-height: 1.45;
    }

    .safety-banner.safe {
        background: var(--color-success-soft);
        color: var(--color-success);
    }

    .safety-banner.caution {
        background: var(--color-warning-soft);
        color: var(--color-warning);
    }

    .safety-banner.unsafe {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    .safety-banner p {
        margin-top: 2px;
        opacity: 0.9;
    }

    .children-section {
        flex: 1;
        min-height: 0;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .children-list {
        overflow-y: auto;
        scrollbar-width: none;
    }

    .children-list::-webkit-scrollbar {
        display: none;
    }

    .child-row {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 0;
        font-size: 12px;
        color: var(--color-text-secondary);
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .child-name {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .child-size {
        font-variant-numeric: tabular-nums;
        color: var(--color-text-muted);
    }

    .detail-actions {
        display: flex;
        gap: 8px;
        padding-top: 10px;
        border-top: 1px solid var(--color-border-subtle);
    }

    .empty {
        text-align: center;
        color: var(--color-text-dim);
        font-size: 12px;
        padding: 24px;
    }
</style>
