<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
</script>

<PreferencesSection
    title="Keyboard Shortcuts"
    description="View and customize keyboard shortcuts"
>
    <div class="shortcuts-list">
        <div class="shortcut-item">
            <span class="shortcut-action">Show/Hide Kova</span>
            <div class="shortcut-keys">
                <kbd class="key">⌘</kbd>
                <kbd class="key">⇧</kbd>
                <kbd class="key">K</kbd>
            </div>
        </div>
        <div class="shortcut-item">
            <span class="shortcut-action">Open Preferences</span>
            <div class="shortcut-keys">
                <kbd class="key">⌘</kbd>
                <kbd class="key">,</kbd>
            </div>
        </div>
        <div class="shortcut-item">
            <span class="shortcut-action">Clear Clipboard</span>
            <div class="shortcut-keys">
                <kbd class="key">⌘</kbd>
                <kbd class="key">⇧</kbd>
                <kbd class="key">X</kbd>
            </div>
        </div>
        <div class="shortcut-item">
            <span class="shortcut-action">Search Clipboard</span>
            <div class="shortcut-keys">
                <kbd class="key">⌘</kbd>
                <kbd class="key">/</kbd>
            </div>
        </div>
        <div class="shortcut-item">
            <span class="shortcut-action">Quit Application</span>
            <div class="shortcut-keys">
                <kbd class="key">⌘</kbd>
                <kbd class="key">Q</kbd>
            </div>
        </div>
    </div>
</PreferencesSection>

<style>
    .shortcuts-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .shortcut-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 12px 16px;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-subtle);
    }

    .shortcut-action {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-primary);
    }

    .shortcut-keys {
        display: flex;
        gap: 4px;
    }

    .key {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 28px;
        height: 28px;
        padding: 0 8px;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-sm);
        font-size: 12px;
        font-weight: 600;
        color: var(--color-text-primary);
        font-family: inherit;
    }
</style>
