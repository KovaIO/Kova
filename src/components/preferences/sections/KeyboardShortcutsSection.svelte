<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import { Trash2 } from "@lucide/svelte";

    type Shortcut = { action: string; keys: string };

    const sections: { title: string; shortcuts: Shortcut[] }[] = [
        {
            title: "Window Manager",
            shortcuts: [
                {
                    action: "Move window to next screen",
                    keys: "Ctrl + Alt + →",
                },
                { action: "Match with another window", keys: "Ctrl + Alt + M" },
                { action: "Auto layout windows", keys: "Ctrl + Alt + L" },
                { action: "Center window", keys: "Ctrl + Alt + C" },
                { action: "Make window 16 by 9", keys: "Ctrl + Alt + 9" },
            ],
        },
        {
            title: "Window Switcher",
            shortcuts: [
                { action: "Open window switcher", keys: "Ctrl + Tab" },
                {
                    action: "Select previous window",
                    keys: "Ctrl + Shift + Tab",
                },
                { action: "Search", keys: "Ctrl + F" },
                { action: "Expand tabs", keys: "Ctrl + Shift + E" },
                { action: "Collapse tabs", keys: "Ctrl + Shift + C" },
            ],
        },
        {
            title: "Clipboard History",
            shortcuts: [
                { action: "Open clipboard history", keys: "Ctrl + Shift + H" },
                {
                    action: "Search clipboard history",
                    keys: "Ctrl + Shift + F",
                },
                { action: "Move up", keys: "↑" },
                { action: "Move down", keys: "↓" },
                { action: "Select item", keys: "Enter" },
                { action: "Paste original", keys: "Ctrl + V" },
                { action: "Paste plain text", keys: "Ctrl + Shift + V" },
                { action: "Paste plain text directly", keys: "Ctrl + Alt + V" },
            ],
        },
        {
            title: "Windows",
            shortcuts: [
                {
                    action: "Open system monitoring dashboard",
                    keys: "Ctrl + Alt + D",
                },
                { action: "Open menubar popover", keys: "Ctrl + Alt + B" },
            ],
        },
    ];
</script>

<PreferencesSection
    title="Keyboard Shortcuts"
    description="View and customize keyboard shortcuts"
>
    {#each sections as section}
        <div class="shortcuts-section">
            <div class="section-title">{section.title}</div>
            <div class="shortcuts-list">
                {#each section.shortcuts as shortcut}
                    <div class="shortcut-item">
                        <span class="shortcut-action">{shortcut.action}</span>
                        <div class="shortcut-keys">
                            <span class="keybind">{shortcut.keys}</span>
                            <button class="trash-button" title="Remove keybind">
                                <Trash2 size={13} />
                            </button>
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {/each}
</PreferencesSection>

<style>
    .shortcuts-section {
        margin-bottom: 24px;
    }
    .shortcuts-section:last-child {
        margin-bottom: 0;
    }

    .section-title {
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--color-text-tertiary);
        margin-bottom: 12px;
    }

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
        align-items: center;
        gap: 8px;
    }

    .keybind {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 4px 10px;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-sm);
        font-size: 12px;
        font-weight: 600;
        color: var(--color-text-primary);
        font-family: inherit;
        letter-spacing: 0.02em;
    }

    .trash-button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 24px;
        height: 24px;
        padding: 0;
        border: none;
        background: transparent;
        color: var(--color-text-tertiary);
        border-radius: 4px;
        transition: all 150ms ease;
    }

    .trash-button:hover {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }
</style>
