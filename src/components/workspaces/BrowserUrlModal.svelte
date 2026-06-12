<script lang="ts">
    export let appName: string = "";
    export let urls: string[] = [];
    export let onsave: (urls: string[]) => void = () => {};
    export let onclose: () => void = () => {};

    let newUrl = "";
    let listEl: HTMLDivElement;

    function addUrl() {
        const trimmed = newUrl.trim();
        if (!trimmed) return;
        if (!urls.includes(trimmed)) {
            urls = [...urls, trimmed];
        }
        newUrl = "";
    }

    function removeUrl(index: number) {
        urls = urls.filter((_, i) => i !== index);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Enter") {
            e.preventDefault();
            addUrl();
        } else if (e.key === "Escape") {
            onclose();
        }
    }

    function handleBackdrop(e: MouseEvent) {
        if (e.target === e.currentTarget) onclose();
    }

    function save() {
        onsave(urls);
        onclose();
    }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="backdrop" on:click={handleBackdrop} on:keydown={handleKeydown} role="dialog" aria-modal="true" tabindex="-1">
    <div class="modal">
        <div class="modal-header">
            <span class="modal-title">Tabs for {appName}</span>
            <button class="close-btn" aria-label="Close" on:click={onclose}>
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
            </button>
        </div>

        <div class="url-input-wrap">
            <input
                class="url-input"
                type="url"
                placeholder="https://example.com"
                bind:value={newUrl}
                on:keydown={handleKeydown}
            />
            <button class="add-btn" on:click={addUrl} disabled={!newUrl.trim()} aria-label="Add URL">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                    <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
                </svg>
            </button>
        </div>

        <div class="url-list" bind:this={listEl}>
            {#if urls.length === 0}
                <div class="empty-msg">No tabs configured — browser will open normally</div>
            {:else}
                {#each urls as url, i}
                    <div class="url-row">
                        <span class="url-text">{url}</span>
                        <button class="url-remove" on:click={() => removeUrl(i)} aria-label="Remove URL">
                            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                                <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                            </svg>
                        </button>
                    </div>
                {/each}
            {/if}
        </div>

        <div class="modal-footer">
            <button class="cancel-btn" on:click={onclose}>Cancel</button>
            <button class="save-btn" on:click={save}>Save</button>
        </div>
    </div>
</div>

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
        backdrop-filter: blur(4px);
    }

    .modal {
        width: 420px;
        max-height: 480px;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-lg);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 16px 0;
        flex-shrink: 0;
    }

    .modal-title {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
    }

    .close-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 24px;
        height: 24px;
        border: none;
        background: transparent;
        color: var(--color-text-dim);
        border-radius: 4px;
        transition: all var(--transition-fast);
    }
    .close-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .url-input-wrap {
        display: flex;
        gap: 6px;
        padding: 12px 16px;
        flex-shrink: 0;
    }

    .url-input {
        flex: 1;
        padding: 8px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-primary);
        font-size: 13px;
        font-family: inherit;
        transition: border-color var(--transition-fast);
    }
    .url-input::placeholder {
        color: var(--color-text-dim);
    }
    .url-input:focus {
        outline: none;
        border-color: var(--color-accent-border);
    }

    .add-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        flex-shrink: 0;
        transition: all var(--transition-fast);
    }
    .add-btn:hover:not(:disabled) {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }
    .add-btn:disabled {
        opacity: 0.4;
        cursor: default;
    }

    .url-list {
        flex: 1;
        overflow-y: auto;
        scrollbar-width: none;
        padding: 0 16px;
        min-height: 80px;
    }
    .url-list::-webkit-scrollbar {
        display: none;
    }

    .empty-msg {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 24px;
        font-size: 12px;
        color: var(--color-text-dim);
        text-align: center;
    }

    .url-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 7px 8px;
        border-radius: var(--radius-sm);
        transition: background var(--transition-fast);
    }
    .url-row:hover {
        background: var(--color-button-bg);
    }

    .url-text {
        font-size: 12.5px;
        color: var(--color-text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        min-width: 0;
    }

    .url-remove {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 20px;
        height: 20px;
        border: none;
        background: transparent;
        color: var(--color-text-dim);
        border-radius: 4px;
        flex-shrink: 0;
        opacity: 0;
        transition: all var(--transition-fast);
    }
    .url-row:hover .url-remove {
        opacity: 1;
    }
    .url-remove:hover {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    .modal-footer {
        display: flex;
        justify-content: flex-end;
        gap: 6px;
        padding: 12px 16px;
        border-top: 1px solid var(--color-border-subtle);
        flex-shrink: 0;
    }

    .cancel-btn {
        padding: 6px 14px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: transparent;
        color: var(--color-text-secondary);
        font-size: 12.5px;
        font-family: inherit;
        transition: all var(--transition-fast);
    }
    .cancel-btn:hover {
        background: var(--color-button-bg);
    }

    .save-btn {
        padding: 6px 14px;
        border-radius: var(--radius-sm);
        border: none;
        background: var(--color-accent);
        color: #fff;
        font-size: 12.5px;
        font-weight: 500;
        font-family: inherit;
        transition: opacity var(--transition-fast);
    }
    .save-btn:hover {
        opacity: 0.85;
    }
</style>
