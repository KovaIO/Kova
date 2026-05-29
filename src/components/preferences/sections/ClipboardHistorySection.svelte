<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import AppPickerModal from "$components/clipboard/AppPickerModal.svelte";
    import type { IgnoredApp } from "$types/preferences";

    let ignoredApps: IgnoredApp[] = [];
    let showPicker = false;
    let clearing = false;

    async function clearHistory() {
        clearing = true;
        // TODO: invoke("clear_clipboard_history")
        await new Promise((r) => setTimeout(r, 800));
        clearing = false;
    }

    function onPick(app: IgnoredApp) {
        if (ignoredApps.some((a) => a.path === app.path)) return;
        ignoredApps = [...ignoredApps, app];
        showPicker = false;
    }

    function removeApp(index: number) {
        ignoredApps = ignoredApps.filter((_, i) => i !== index);
    }
</script>

{#if showPicker}
    <AppPickerModal onpick={onPick} onclose={() => (showPicker = false)} />
{/if}

<PreferencesSection
    title="Clipboard History"
    description="Manage clipboard history and behavior"
>
    <PreferenceItem
        label="Enable clipboard history"
        description="Keep track of items copied to clipboard"
    >
        <Toggle label="" id="enable-clipboard" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="History limit"
        description="Maximum number of items to keep in history"
    >
        <select class="select">
            <option>10 items</option>
            <option selected>25 items</option>
            <option>50 items</option>
            <option>100 items</option>
            <option>Unlimited</option>
        </select>
    </PreferenceItem>

    <PreferenceItem
        label="Ignore passwords"
        description="Don't save password fields to clipboard history"
    >
        <Toggle label="" id="ignore-passwords" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="Clear history"
        description="Permanently delete all clipboard history entries"
    >
        <button
            class="clear-btn"
            class:clearing
            on:click={clearHistory}
            disabled={clearing}
        >
            {#if clearing}
                <svg
                    class="spin"
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                >
                    <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                </svg>
                Clearing…
            {:else}
                Clear now
            {/if}
        </button>
    </PreferenceItem>

    <div class="ignored-section">
        <div class="ignored-header">
            <span class="ignored-title">Ignored apps</span>
            <span class="ignored-hint"
                >Clipboard won't be tracked for these apps</span
            >
        </div>

        <div class="ignored-list">
            {#if ignoredApps.length === 0}
                <div class="empty-state">No ignored apps</div>
            {:else}
                {#each ignoredApps as app, i}
                    <div class="ignored-item">
                        {#if app.icon}
                            <img
                                class="app-icon"
                                src={`data:image/png;base64,${app.icon}`}
                                alt=""
                            />
                        {:else}
                            <div class="app-icon placeholder"></div>
                        {/if}
                        <div class="app-info">
                            <span class="app-name">{app.name}</span>
                            <span class="app-path">{app.path}</span>
                        </div>
                        <button
                            class="remove-btn"
                            title="Remove"
                            on:click={() => removeApp(i)}
                        >
                            <svg
                                width="12"
                                height="12"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2.5"
                                stroke-linecap="round"
                            >
                                <line x1="18" y1="6" x2="6" y2="18" />
                                <line x1="6" y1="6" x2="18" y2="18" />
                            </svg>
                        </button>
                    </div>
                {/each}
            {/if}
        </div>

        <button class="add-btn" on:click={() => (showPicker = true)}>
            <svg
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Add app
        </button>
    </div>
</PreferencesSection>

<style>
    .select {
        padding: 8px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-primary);
        font-size: 13px;
        min-width: 140px;
    }
    .select:hover {
        background: var(--color-button-bg-hover);
    }
    .select:focus {
        outline: none;
        border-color: var(--color-accent-border);
    }

    .clear-btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 8px 14px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-danger-soft);
        background: transparent;
        color: var(--color-danger);
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        transition: all var(--transition-fast);
    }
    .clear-btn:hover {
        background: var(--color-danger-soft);
    }
    .clear-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    .clear-btn.clearing {
        color: var(--color-text-muted);
        border-color: var(--color-border-medium);
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
    .spin {
        animation: spin 700ms linear infinite;
    }

    .ignored-section {
        display: flex;
        flex-direction: column;
        gap: 10px;
        padding-top: 4px;
    }

    .ignored-header {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .ignored-title {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-primary);
    }

    .ignored-hint {
        font-size: 12px;
        color: var(--color-text-muted);
    }

    .ignored-list {
        display: flex;
        flex-direction: column;
        border: 1px solid var(--color-border-subtle);
        border-radius: var(--radius-sm);
        background: var(--color-surface-elevated);
        overflow: hidden;
        min-height: 44px;
    }

    .empty-state {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 20px;
        font-size: 12px;
        color: var(--color-text-dim);
    }

    .ignored-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 12px;
        border-bottom: 1px solid var(--color-border-subtle);
        transition: background var(--transition-fast);
    }
    .ignored-item:last-child {
        border-bottom: none;
    }
    .ignored-item:hover {
        background: var(--color-button-bg);
    }

    .app-icon {
        width: 24px;
        height: 24px;
        flex-shrink: 0;
        border-radius: 4px;
        object-fit: contain;
    }

    .app-icon.placeholder {
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-subtle);
    }

    .app-info {
        display: flex;
        flex-direction: column;
        gap: 2px;
        flex: 1;
        min-width: 0;
    }

    .app-name {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-secondary);
    }

    .app-path {
        font-size: 11px;
        color: var(--color-text-dim);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .remove-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        border: none;
        background: transparent;
        color: var(--color-text-dim);
        border-radius: 4px;
        opacity: 0;
        transition: all var(--transition-fast);
        flex-shrink: 0;
    }
    .ignored-item:hover .remove-btn {
        opacity: 1;
    }
    .remove-btn:hover {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    .add-btn {
        display: inline-flex;
        align-items: center;
        gap: 7px;
        align-self: flex-start;
        padding: 8px 14px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        transition: all var(--transition-fast);
    }
    .add-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
        border-color: var(--color-border-strong);
    }
</style>
