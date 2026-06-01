<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import ShortcutCapturePopover from "../ShortcutCapturePopover.svelte";
    import { Trash2 } from "@lucide/svelte";

    import {
        actionToSnake,
        CATALOG_SECTIONS,
        shortcutsForApi,
    } from "$utils/keyboard-shortcuts";
    import { updateShortcuts } from "$services/preferences";
    import { preferences } from "$stores/preferences";
    import type { Shortcut } from "$types/preferences";

    type RecordingState = {
        action: string;
        label: string;
        anchor: DOMRect;
    };

    let recording: RecordingState | null = null;
    let savingAction: string | null = null;

    $: savedShortcuts = $preferences?.shortcuts ?? [];
    $: keysByAction = Object.fromEntries(
        savedShortcuts.map((shortcut) => [
            actionToSnake(shortcut.action),
            shortcut.keys,
        ]),
    );

    function openRecorder(action: string, label: string, event: MouseEvent) {
        const target = event.currentTarget as HTMLElement;
        recording = {
            action,
            label,
            anchor: target.getBoundingClientRect(),
        };
    }

    function buildUpdatedShortcuts(
        action: string,
        keys: string | null,
    ): Shortcut[] {
        const snake = actionToSnake(action);
        const others = savedShortcuts.filter(
            (shortcut) => actionToSnake(shortcut.action) !== snake,
        );

        if (!keys) {
            return others;
        }

        return [...others, { action: snake, keys }];
    }

    async function persistShortcuts(action: string, next: Shortcut[]) {
        savingAction = action;
        try {
            await updateShortcuts(next);
        } finally {
            savingAction = null;
        }
    }

    async function confirmRecording(keys: string) {
        if (!recording) return;

        const { action } = recording;
        recording = null;
        await persistShortcuts(action, buildUpdatedShortcuts(action, keys));
    }

    async function removeShortcut(action: string) {
        await persistShortcuts(action, buildUpdatedShortcuts(action, null));
    }

    function cancelRecording() {
        recording = null;
    }

    $: shortcutsForCapture = shortcutsForApi(savedShortcuts);
</script>

{#if recording}
    <ShortcutCapturePopover
        anchor={recording.anchor}
        actionLabel={recording.label}
        editingAction={recording.action}
        shortcuts={shortcutsForCapture}
        onconfirm={confirmRecording}
        oncancel={cancelRecording}
    />
{/if}

<PreferencesSection
    title="Keyboard Shortcuts"
    description="View and customize keyboard shortcuts"
>
    {#each CATALOG_SECTIONS as section (section.title)}
        <div class="shortcuts-section">
            <div class="section-title">{section.title}</div>
            <div class="shortcuts-list">
                {#each section.shortcuts as entry (entry.action)}
                    <div class="shortcut-item">
                        <span class="shortcut-action">{entry.label}</span>
                        <div class="shortcut-keys">
                            <button
                                type="button"
                                class="keybind"
                                class:keybind-empty={!keysByAction[
                                    actionToSnake(entry.action)
                                ]}
                                disabled={savingAction === entry.action}
                                on:click={(event) =>
                                    openRecorder(
                                        entry.action,
                                        entry.label,
                                        event,
                                    )}
                            >
                                {keysByAction[actionToSnake(entry.action)] ??
                                    "Click to set"}
                            </button>
                            {#if keysByAction[actionToSnake(entry.action)]}
                                <button
                                    type="button"
                                    class="trash-button"
                                    title="Remove shortcut"
                                    disabled={savingAction === entry.action}
                                    on:click={() =>
                                        removeShortcut(entry.action)}
                                >
                                    <Trash2 size={13} />
                                </button>
                            {/if}
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
        min-width: 96px;
        padding: 4px 10px;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-sm);
        font-size: 12px;
        font-weight: 600;
        color: var(--color-text-primary);
        font-family: inherit;
        letter-spacing: 0.02em;
        cursor: pointer;
        transition:
            border-color 150ms ease,
            background 150ms ease;
    }

    .keybind:hover:not(:disabled) {
        border-color: var(--color-border-strong, var(--color-border-medium));
        background: var(--color-surface-hover, var(--color-button-bg));
    }

    .keybind:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .keybind-empty {
        color: var(--color-text-tertiary);
        font-weight: 500;
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
        cursor: pointer;
    }

    .trash-button:hover:not(:disabled) {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    .trash-button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
