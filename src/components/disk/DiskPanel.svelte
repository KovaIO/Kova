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
    import { safetyLabel } from "$types/disk";
    import { formatBytes } from "$utils/format";

    type View = "ready" | "scanning" | "results" | "detail";

    let view: View = $state("ready");
    let {
        volume = $bindable(null),
    }: {
        volume?: DiskVolumeInfo | null;
    } = $props();
    let preview: ScanPreview | null = $state(null);
    let result: DiskScanResult | null = $state(null);
    let detail: DiskItemDetail | null = $state(null);
    let scanProgress = $state(0);
    let scanMessage = $state("");
    let expandedCategories = $state(new Set<DiskCategory>());
    let confirmDeleteAll = $state(false);
    let confirmDeleteId: string | null = $state(null);
    let deleting = $state(false);
    let unlisten: UnlistenFn | undefined;

    let reclaimable = $derived(
        (result as DiskScanResult | null)?.categories
            .flatMap((g) => g.items)
            .filter((i) => i.action === "delete" && i.safety !== "unsafe")
            .reduce((sum, i) => sum + i.size_bytes, 0) ?? 0,
    );

    function isDeletable(item: DiskScanItem): boolean {
        return item.action === "delete" && item.safety !== "unsafe";
    }

    async function loadInitial() {
        try {
            preview = await fetchDiskScanPreview();
            // Don't load scan results - start in ready state
            view = "ready";
        } catch {
            /* empty */
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
        deleting = true;
        try {
            await deleteDiskItems([id]);
        } catch {
            /* some files may be locked */
        }
        confirmDeleteId = null;
        result = await fetchDiskScanResult();
        volume = await fetchDiskVolumeInfo();
        deleting = false;
        if (detail?.item.id === id) {
            detail = null;
            view = result?.categories.length ? "results" : "ready";
        }
        if (!result?.categories.length) view = "ready";
    }

    async function deleteAll() {
        deleting = true;
        try {
            await deleteAllDiskItems();
        } catch {
            /* some files may be locked */
        }
        confirmDeleteAll = false;
        result = await fetchDiskScanResult();
        volume = await fetchDiskVolumeInfo();
        deleting = false;
        view = result?.categories.length ? "results" : "ready";
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
    {#if view === "detail" && detail}
        <div class="detail-view">
            <div class="card detail-card">
                <div class="detail-head">
                    <button class="back-btn" onclick={() => (view = "results")}>
                        <ArrowLeft size={14} />
                    </button>
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

                {#if isDeletable(detail.item)}
                    <div class="detail-actions">
                        {#if confirmDeleteId === detail.item.id}
                            <button
                                type="button"
                                class="btn danger"
                                disabled={deleting}
                                onclick={() => void deleteOne(detail!.item.id)}
                            >
                                {deleting ? "Deleting…" : "Confirm delete"}
                            </button>
                            <button
                                type="button"
                                class="btn ghost"
                                disabled={deleting}
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
                <div class="ready-content">
                    <HardDrive size={18} />
                    <h3>Ready to clean</h3>
                    <p>
                        Scan {volume?.mount_path ?? "your disk"} for reclaimable cache
                        and temporary files.
                    </p>

                    <div class="preview-badges">
                        {#each preview?.categories ?? [] as cat}
                            <div class="preview-badge">
                                {#if categoryIcon(cat.category)}
                                    {@const Icon = categoryIcon(cat.category)}
                                    <Icon size={12} />
                                {/if}
                                <span class="badge-name">{cat.label}</span>
                            </div>
                        {/each}
                    </div>
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
                                            {#if isDeletable(item)}
                                                {#if confirmDeleteId === item.id}
                                                    <button
                                                        type="button"
                                                        class="icon-btn confirm"
                                                        disabled={deleting}
                                                        onclick={() =>
                                                            void deleteOne(
                                                                item.id,
                                                            )}
                                                    >
                                                        {deleting ? "…" : "OK"}
                                                    </button>
                                                {:else}
                                                    <button
                                                        type="button"
                                                        class="icon-btn"
                                                        title="Delete"
                                                        disabled={deleting}
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
                                disabled={deleting}
                                onclick={() => void deleteAll()}
                            >
                                {deleting ? "Deleting…" : "Confirm delete all"}
                            </button>
                            <button
                                type="button"
                                class="btn ghost"
                                disabled={deleting}
                                onclick={() => (confirmDeleteAll = false)}
                            >
                                Cancel
                            </button>
                        </div>
                    {:else}
                        <button
                            type="button"
                            class="btn ghost"
                            disabled={deleting}
                            onclick={() => void handleScan()}
                        >
                            Scan again
                        </button>
                        <button
                            type="button"
                            class="btn danger"
                            disabled={deleting}
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
        gap: 6px;
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

    .ready-card,
    .scan-card,
    .results-card,
    .detail-card {
        padding: 10px 12px;
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .ready-card {
        align-items: center;
        text-align: center;
    }

    .ready-content {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 8px;
        min-height: 0;
        color: var(--color-text-secondary);
    }

    .ready-content h3 {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin-bottom: 2px;
    }

    .ready-content p {
        font-size: 11px;
        line-height: 1.4;
        color: var(--color-text-muted);
    }

    .preview-badges {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        gap: 8px;
        margin-top: 6px;
    }

    .preview-badge {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 5px 10px;
        background: var(--color-border-subtle);
        border-radius: var(--radius-sm);
        font-size: 11px;
        color: var(--color-text-secondary);
    }

    .badge-name {
        font-weight: 500;
        color: var(--color-text-muted);
    }

    .scan-card {
        align-items: center;
        justify-content: center;
        gap: 10px;
    }

    .scan-animation {
        position: relative;
        width: 40px;
        height: 40px;
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
        font-size: 11.5px;
        color: var(--color-text-secondary);
    }

    .progress-track {
        width: 100%;
        max-width: 200px;
        height: 3px;
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
        gap: 6px;
        padding: 6px 8px;
        margin-bottom: 8px;
        border-radius: var(--radius-sm);
        background: var(--color-success-soft);
        color: var(--color-success);
        font-size: 11px;
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
        gap: 6px;
        padding: 7px 4px;
        border: none;
        background: transparent;
        color: var(--color-text-secondary);
        font: inherit;
        font-size: 12px;
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
        font-size: 11px;
    }

    .items-list {
        padding: 0 4px 6px 18px;
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
        gap: 6px;
        padding: 5px 4px;
        border: none;
        background: transparent;
        color: inherit;
        font: inherit;
        font-size: 11.5px;
        text-align: left;
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
        font-size: 11px;
        color: var(--color-text-muted);
        font-variant-numeric: tabular-nums;
    }

    .icon-btn {
        width: 24px;
        height: 24px;
        border: none;
        border-radius: 5px;
        background: transparent;
        color: var(--color-text-dim);
        display: grid;
        place-items: center;
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
        font-size: 10px;
        font-weight: 600;
        color: var(--color-danger);
    }

    .results-footer {
        display: flex;
        flex-direction: column;
        gap: 6px;
        padding-top: 8px;
        border-top: 1px solid var(--color-border-subtle);
        flex-shrink: 0;
    }

    .footer-actions {
        display: flex;
        gap: 6px;
    }

    .warning-text {
        font-size: 11px;
        color: var(--color-warning);
        line-height: 1.4;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        padding: 7px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 12px;
        font-weight: 500;
        font-family: inherit;
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
        margin-top: auto;
    }

    .detail-view {
        display: flex;
        flex-direction: column;
        gap: 6px;
        flex: 1;
        min-height: 0;
    }

    .back-btn {
        display: inline-flex;
        align-items: center;
        gap: 5px;
        align-self: flex-start;
        padding: 4px 8px;
        border: none;
        background: transparent;
        color: var(--color-text-muted);
        font-size: 11px;
        font-family: inherit;
        border-radius: var(--radius-sm);
    }

    .back-btn:hover {
        background: var(--color-button-bg);
        color: var(--color-text-primary);
    }

    .detail-head {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 8px;
    }

    .detail-icon {
        width: 32px;
        height: 32px;
        border-radius: 7px;
        background: var(--color-track-fill);
        display: grid;
        place-items: center;
        color: var(--color-text-secondary);
    }

    .detail-titles h2 {
        font-size: 13px;
        font-weight: 600;
        margin-bottom: 2px;
    }

    .detail-path {
        font-size: 10px;
        color: var(--color-text-dim);
        word-break: break-all;
    }

    .meta-grid {
        display: flex;
        flex-direction: column;
        gap: 4px;
        margin-bottom: 8px;
    }

    .meta-row {
        display: flex;
        justify-content: space-between;
        font-size: 11px;
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
        gap: 8px;
        padding: 8px 10px;
        border-radius: var(--radius-sm);
        margin-bottom: 8px;
        font-size: 11px;
        line-height: 1.4;
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
        gap: 6px;
        padding: 4px 0;
        font-size: 11px;
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
        gap: 6px;
        padding-top: 8px;
        border-top: 1px solid var(--color-border-subtle);
    }
</style>
