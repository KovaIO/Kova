<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { Cpu, Image as ImageIcon, Terminal } from "@lucide/svelte";
    import type { ClipboardItem } from "$types/clipboard";
    import { slide } from "svelte/transition";
    import {
        formatClipboardTime,
        formatFileSize,
        imageFolder,
        isRasterIcon,
        isUrl,
        previewText,
    } from "$types/clipboard";
    import {
        copyClipboardItem,
        deleteClipboardItem,
        openClipboardUrl,
        pasteClipboardItem,
        pastePlainClipboardItem,
        previewClipboardItem,
        revealClipboardItem,
    } from "$services/clipboard";

    interface Props {
        item: ClipboardItem;
        expanded?: boolean;
        highlight?: string;
        ontoggle?: () => void;
        onchanged?: () => void;
    }

    let { item, expanded = false, highlight = "", ontoggle, onchanged }: Props = $props();

    let thumbFailed = $state(false);

    let preview = $derived(previewText(item, expanded ? 800 : 180));
    let timeLabel = $derived(formatClipboardTime(item.created_at));
    let imageSrc = $derived(
        item.image_path && !thumbFailed
            ? convertFileSrc(item.image_path)
            : null,
    );
    let showUrlAction = $derived(isUrl(item));
    let showFileActions = $derived(
        item.content_type === "image" && !!item.image_path,
    );

    let highlightedPreview = $derived(highlightText(preview || "Empty text", highlight));

    function run(action: () => Promise<void>) {
        void action().then(() => onchanged?.());
    }

    function highlightText(text: string, query: string): string {
        if (!query.trim()) return escapeHtml(text);
        const escaped = escapeHtml(text);
        const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        return escaped.replace(
            new RegExp(`(${escapedQuery})`, "gi"),
            "<mark>$1</mark>"
        );
    }

    function escapeHtml(text: string): string {
        return text
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }
</script>

<div class="card" class:expanded>
    <button
        type="button"
        class="main"
        onclick={ontoggle}
        aria-expanded={expanded}
    >
        {#if item.content_type === "image"}
            <div class="image-head">
                {#if imageSrc}
                    <img
                        class="thumb"
                        src={imageSrc}
                        alt=""
                        onerror={() => (thumbFailed = true)}
                    />
                {:else}
                    <div class="thumb placeholder">
                        <ImageIcon size={16} />
                    </div>
                {/if}
                <div class="image-meta">
                    <span class="image-title"
                        >{item.image_filename ?? "Image"}</span
                    >
                    <span class="image-sub">{imageFolder(item.image_path)}</span
                    >
                </div>
            </div>

            {#if expanded}
                <div class="meta-grid">
                    <div class="meta-row">
                        <span>Size</span>
                        <span>{formatFileSize(item.image_size)}</span>
                    </div>
                    <div class="meta-row">
                        <span>Resolution</span>
                        <span>
                            {#if item.image_width && item.image_height}
                                {item.image_width}×{item.image_height}
                            {:else}
                                —
                            {/if}
                        </span>
                    </div>
                </div>
            {/if}
        {:else}
            <div class="text-preview" class:expanded>
                {@html highlightedPreview}
            </div>
        {/if}

        <div class="footer">
            {#if item.source_app_icon === "system"}
                <div class="app-icon fallback"><Cpu size={12} /></div>
            {:else if item.source_app_icon === "terminal"}
                <div class="app-icon fallback"><Terminal size={12} /></div>
            {:else if isRasterIcon(item.source_app_icon)}
                <img
                    class="app-icon"
                    src={`data:image/png;base64,${item.source_app_icon}`}
                    alt=""
                />
            {:else}
                <div class="app-icon placeholder"></div>
            {/if}
            <span class="source">{item.source_app ?? "Unknown"}</span>
            <span class="time">{timeLabel}</span>
        </div>
    </button>

    {#if expanded}
        <div class="actions" transition:slide={{ duration: 180, axis: "y" }}>
            <button
                type="button"
                class="action"
                onclick={() => run(() => pasteClipboardItem(item.id))}
            >
                <span>Paste</span><kbd>O</kbd>
            </button>
            <button
                type="button"
                class="action"
                onclick={() => run(() => copyClipboardItem(item.id))}
            >
                <span>Copy</span><kbd>C</kbd>
            </button>
            <button
                type="button"
                class="action"
                onclick={() => run(() => pastePlainClipboardItem(item.id))}
            >
                <span>Paste plain text</span><kbd>P</kbd>
            </button>
            {#if showUrlAction}
                <button
                    type="button"
                    class="action"
                    onclick={() => run(() => openClipboardUrl(item.id))}
                >
                    <span>Open URL</span><kbd>↵</kbd>
                </button>
            {/if}
            {#if showFileActions}
                <button
                    type="button"
                    class="action"
                    onclick={() => run(() => revealClipboardItem(item.id))}
                >
                    <span>Show in Explorer</span><kbd>F</kbd>
                </button>
                <button
                    type="button"
                    class="action"
                    onclick={() => run(() => previewClipboardItem(item.id))}
                >
                    <span>Quick look</span><kbd>Space</kbd>
                </button>
            {/if}
            <button
                type="button"
                class="action danger"
                onclick={() => run(() => deleteClipboardItem(item.id))}
            >
                <span>Delete</span><kbd>Del</kbd>
            </button>
        </div>
    {/if}
</div>

<style>
    :global(mark) {
        background: #ffd700;
        color: #000;
        border-radius: 2px;
        padding: 0 1px;
    }

    .card {
        border: 2px solid transparent;
        border-radius: var(--radius-md);
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        overflow: hidden;
        transition:
            border-color var(--transition-fast),
            background var(--transition-fast);
    }

    .main {
        width: 100%;
        padding: 0;
        border: none;
        background: transparent;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
    }

    .text-preview {
        padding: 12px 14px 10px;
        font-size: 12.5px;
        line-height: 1.45;
        color: var(--color-text-primary);
        white-space: pre-wrap;
        word-break: break-word;
        max-height: 88px;
        overflow: hidden;
        border-bottom: 1px solid var(--color-border-subtle);
        transition: max-height 200ms cubic-bezier(0.4, 0, 0.2, 1);
    }

    .text-preview.expanded {
        max-height: 220px;
        overflow: auto;
        scrollbar-width: none;
        -ms-overflow-style: none;
    }

    .text-preview.expanded::-webkit-scrollbar {
        display: none;
    }

    .image-head {
        display: flex;
        gap: 10px;
        padding: 12px 14px 10px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .thumb {
        width: 52px;
        height: 40px;
        object-fit: cover;
        border-radius: 6px;
        flex-shrink: 0;
        background: var(--color-track-fill);
    }

    .thumb.placeholder {
        display: grid;
        place-items: center;
        color: var(--color-text-tertiary);
    }

    .image-meta {
        display: flex;
        flex-direction: column;
        gap: 3px;
        min-width: 0;
        justify-content: center;
    }

    .image-title {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .image-sub {
        font-size: 11.5px;
        color: var(--color-text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .meta-grid {
        display: flex;
        flex-direction: column;
        gap: 4px;
        padding: 0 14px 10px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .meta-row {
        display: flex;
        justify-content: space-between;
        gap: 12px;
        font-size: 12px;
        color: var(--color-text-muted);
    }

    .meta-row span:last-child {
        color: var(--color-text-secondary);
        font-variant-numeric: tabular-nums;
    }

    .footer {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
    }

    .app-icon {
        width: 18px;
        height: 18px;
        border-radius: 4px;
        object-fit: contain;
        flex-shrink: 0;
    }

    .app-icon.placeholder,
    .app-icon.fallback {
        display: grid;
        place-items: center;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-subtle);
        color: var(--color-text-secondary);
    }

    .source {
        flex: 1;
        font-size: 12px;
        color: var(--color-text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .time {
        font-size: 12px;
        color: var(--color-text-muted);
        font-variant-numeric: tabular-nums;
        flex-shrink: 0;
    }

    .actions {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 4px;
        border-top: 1px solid var(--color-border-subtle);
        background: rgba(0, 0, 0, 0.12);
    }

    .action {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 8px 10px;
        border: none;
        border-radius: var(--radius-sm);
        background: transparent;
        color: var(--color-text-secondary);
        font-size: 13px;
        font-family: inherit;
        cursor: pointer;
        transition: background var(--transition-fast);
    }

    .action:hover {
        background: var(--color-button-bg);
        color: var(--color-text-primary);
    }

    .action.danger:hover {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    kbd {
        font-size: 11px;
        color: var(--color-text-dim);
        border: 1px solid var(--color-border-medium);
        border-radius: 6px;
        padding: 2px 6px;
        background: var(--color-track-base);
        font-family: inherit;
    }
</style>
