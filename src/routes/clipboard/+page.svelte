<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { Search } from "@lucide/svelte";
    import { fade } from "svelte/transition";
    import { flip } from "svelte/animate";
    import ClipboardHistoryItem from "$components/clipboard/ClipboardHistoryItem.svelte";
    import {
        copyClipboardItem,
        deleteClipboardItem,
        fetchClipboardHistory,
        openClipboardUrl,
        pasteClipboardItem,
        pastePlainClipboardItem,
        previewClipboardItem,
        revealClipboardItem,
    } from "$services/clipboard";
    import type { ClipboardItem } from "$types/clipboard";
    import { isUrl } from "$types/clipboard";
    import WindowAnimation from "$components/WindowAnimation.svelte";

    let items: ClipboardItem[] = $state([]);
    let expandedId: number | null = $state(null);
    let search = $state("");
    let searchFocused = $state(false);
    let loading = $state(true);

    let searchInput: HTMLInputElement | undefined = $state();
    let listEl: HTMLDivElement | undefined = $state();
    let unlisten: UnlistenFn | undefined;
    let unlistenFocus: UnlistenFn | undefined;

    let filteredItems = $derived.by(() => {
        const q = search.trim().toLowerCase();
        if (!q) return items;
        return items.filter(
            (item) =>
                item.text_content?.toLowerCase().includes(q) ||
                item.source_app?.toLowerCase().includes(q),
        );
    });

    let expandedItem = $derived(
        expandedId === null
            ? null
            : (filteredItems.find((item) => item.id === expandedId) ?? null),
    );

    let expandedIndex = $derived(
        expandedId === null
            ? -1
            : filteredItems.findIndex((item) => item.id === expandedId),
    );

    async function hideWindow() {
        await getCurrentWindow().hide();
    }

    async function loadHistory() {
        loading = true;
        try {
            items = await fetchClipboardHistory();
            if (
                expandedId !== null &&
                !items.some((item) => item.id === expandedId)
            ) {
                expandedId = null;
            }
        } finally {
            loading = false;
        }
    }

    function toggleItem(id: number) {
        expandedId = expandedId === id ? null : id;
    }

    function selectByOffset(offset: number) {
        if (filteredItems.length === 0) return;
        const base = expandedIndex < 0 ? 0 : expandedIndex;
        const next =
            (base + offset + filteredItems.length) % filteredItems.length;
        expandedId = filteredItems[next]?.id ?? null;
        scrollExpandedIntoView();
    }

    function scrollExpandedIntoView() {
        if (!listEl || expandedId === null) return;
        const el = listEl.querySelector(`[data-id="${expandedId}"]`);
        el?.scrollIntoView({ block: "nearest" });
    }

    async function runOnExpanded(action: (id: number) => Promise<void>) {
        if (expandedId === null) return;
        await action(expandedId);
        await loadHistory();
    }

    function onKeydown(event: KeyboardEvent) {
        if (event.key === "s" || event.key === "S") {
            if (!searchFocused) {
                event.preventDefault();
                searchInput?.focus();
            }
            return;
        }

        if (event.key === "ArrowDown") {
            event.preventDefault();
            selectByOffset(1);
            return;
        }

        if (event.key === "ArrowUp") {
            event.preventDefault();
            selectByOffset(-1);
            return;
        }

        if (expandedId === null) {
            if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                if (filteredItems[0]) expandedId = filteredItems[0].id;
            }
            return;
        }

        if (searchFocused && event.key !== "Escape") return;

        if (event.key === "o" || event.key === "O") {
            event.preventDefault();
            void runOnExpanded(pasteClipboardItem);
            return;
        }

        if (event.key === "c" || event.key === "C") {
            event.preventDefault();
            void runOnExpanded(copyClipboardItem);
            return;
        }

        if (event.key === "p" || event.key === "P") {
            event.preventDefault();
            void runOnExpanded(pastePlainClipboardItem);
            return;
        }

        if (event.key === "Enter") {
            event.preventDefault();
            if (expandedItem && isUrl(expandedItem)) {
                void runOnExpanded(openClipboardUrl);
            } else {
                void runOnExpanded(pasteClipboardItem);
            }
            return;
        }

        if (event.key === "f" || event.key === "F") {
            if (expandedItem?.content_type === "image") {
                event.preventDefault();
                void runOnExpanded(revealClipboardItem);
            }
            return;
        }

        if (event.key === " ") {
            if (expandedItem?.content_type === "image") {
                event.preventDefault();
                void runOnExpanded(previewClipboardItem);
            }
            return;
        }

        if (event.key === "Delete" || event.key === "Backspace") {
            if (searchFocused && search.length > 0) return;
            event.preventDefault();
            void runOnExpanded(deleteClipboardItem);
        }
    }

    function resetPanel() {
        expandedId = null;
        search = "";
        searchFocused = false;
    }

    onMount(() => {
        void loadHistory();

        (async () => {
            unlisten = await listen("clipboard-history-updated", () => {
                void loadHistory();
            });

            const win = getCurrentWindow();

            unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
                if (!focused) {
                    resetPanel();
                }
            });
        })();

        window.addEventListener("keydown", onKeydown);
    });

    onDestroy(() => {
        unlisten?.();
        unlistenFocus?.();
        window.removeEventListener("keydown", onKeydown);
    });
</script>

<WindowAnimation>
    <div
        class="page"
        role="presentation"
        onclick={(e) => {
            if (e.target === e.currentTarget) hideWindow();
        }}
    >
        <div class="panel">
            <div class="search-card">
                <Search class="search-icon" size={14} />
                <input
                    bind:this={searchInput}
                    class="search-input"
                    type="search"
                    placeholder="Press S to search"
                    bind:value={search}
                    onfocus={() => (searchFocused = true)}
                    onblur={() => (searchFocused = false)}
                />
            </div>

            <div class="list-card" bind:this={listEl}>
                {#if loading}
                    <p class="empty" transition:fade={{ duration: 120 }}>
                        Loading history…
                    </p>
                {:else if filteredItems.length === 0}
                    <p class="empty" transition:fade={{ duration: 120 }}>
                        {search.trim()
                            ? `No results for "${search.trim()}"`
                            : "Copy something to get started"}
                    </p>
                {:else}
                    <div class="list">
                        {#each filteredItems as item (item.id)}
                            <div
                                class="list-item"
                                data-id={item.id}
                                animate:flip={{ duration: 200 }}
                                in:fade={{ duration: 120 }}
                                out:fade={{ duration: 80 }}
                            >
                                <ClipboardHistoryItem
                                    {item}
                                    expanded={item.id === expandedId}
                                    highlight={search.trim()}
                                    ontoggle={() => toggleItem(item.id)}
                                    onchanged={() => void loadHistory()}
                                />
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </div>
</WindowAnimation>

<style>
    .page {
        height: 100vh;
        padding: 10px;
        box-sizing: border-box;
        background: transparent;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .panel {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
        max-height: 100%;
    }

    .search-card {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 0 14px;
        height: 42px;
        flex-shrink: 0;
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        color: var(--color-text-tertiary);
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
        display: flex;
        flex-direction: column;
        min-height: 0;
        overflow-y: auto;
        padding: 4px 0 6px 0;
        overscroll-behavior: contain;
        scrollbar-width: none;
        -ms-overflow-style: none;
    }

    .list-card::-webkit-scrollbar {
        display: none;
        width: 0;
        height: 0;
    }

    .list {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .empty {
        text-align: center;
        background: var(--color-main-bg);
        color: var(--color-text-dim);
        font-size: 12px;
        padding: 28px 12px;
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
    }
</style>
