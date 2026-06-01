<script lang="ts">
    import { onMount, tick } from "svelte";
    import { fade, scale } from "svelte/transition";
    import { backOut } from "svelte/easing";

    import {
        MODIFIER_KEYS,
        partDisplayLabel,
        partsFromKeyboardEvent,
        previewPartsFromEvent,
        type ShortcutValidationResult,
        validateShortcutCapture,
    } from "$utils/keyboard-shortcuts";
    import type { Shortcut } from "$types/preferences";

    export let anchor: DOMRect;
    export let actionLabel: string;
    export let shortcuts: Shortcut[] = [];
    export let editingAction: string;
    export let onconfirm: (keys: string) => void = () => {};
    export let oncancel: () => void = () => {};

    const KEY_POP_MS = 220;
    const HOLD_AFTER_CAPTURE_MS = 140;
    const SHELL_TRANSITION_MS = 220;

    const keyPop = {
        start: 0.5,
        opacity: 0,
        duration: KEY_POP_MS,
        easing: backOut,
    };

    const backdropFade = { duration: SHELL_TRANSITION_MS };

    const shellPop = {
        start: 0.86,
        opacity: 0,
        duration: SHELL_TRANSITION_MS,
        easing: backOut,
    };

    let previewParts: string[] = [];
    let error = "";
    let popoverEl: HTMLDivElement;
    let showShell = true;
    let isClosing = false;

    type CloseReason = { type: "confirm"; keys: string } | { type: "cancel" };
    let closeReason: CloseReason | null = null;

    $: anchorStyle = `
        top: ${Math.max(8, anchor.top - 8)}px;
        left: ${anchor.left + anchor.width / 2}px;
    `;

    function wait(ms: number): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }

    async function completeCapture(keys: string) {
        if (isClosing) return;
        isClosing = true;
        closeReason = { type: "confirm", keys };

        await tick();
        await wait(KEY_POP_MS + HOLD_AFTER_CAPTURE_MS);
        showShell = false;
    }

    function requestDismiss() {
        if (isClosing) return;
        isClosing = true;
        closeReason = { type: "cancel" };
        showShell = false;
    }

    function handleShellOutroEnd() {
        if (closeReason?.type === "confirm") {
            onconfirm(closeReason.keys);
            return;
        }
        oncancel();
    }

    function handleKeydown(event: KeyboardEvent) {
        if (!showShell || isClosing) return;

        event.preventDefault();
        event.stopPropagation();

        if (event.key === "Escape") {
            requestDismiss();
            return;
        }

        previewParts = previewPartsFromEvent(event);

        if (MODIFIER_KEYS.has(event.key)) {
            error = "";
            return;
        }

        const parts = partsFromKeyboardEvent(event);
        if (!parts) {
            error = "";
            return;
        }

        const result: ShortcutValidationResult = validateShortcutCapture(
            parts,
            shortcuts,
            editingAction,
        );

        if (!result.ok) {
            error = result.message;
            return;
        }

        error = "";
        void completeCapture(result.formatted);
    }

    onMount(() => {
        popoverEl?.focus();
    });
</script>

<svelte:window on:keydown|capture={handleKeydown} />

{#if showShell}
    <div
        class="backdrop"
        in:fade={backdropFade}
        out:fade={backdropFade}
        on:click|self={requestDismiss}
        role="presentation"
    >
        <div class="popover-anchor" style={anchorStyle}>
            <div
                bind:this={popoverEl}
                class="popover"
                class:popover-error={Boolean(error)}
                role="dialog"
                aria-modal="true"
                aria-label={actionLabel}
                tabindex="-1"
                in:scale={shellPop}
                out:scale={shellPop}
                on:outroend={handleShellOutroEnd}
            >
                <div class="combo">
                    {#each previewParts as part, index (index)}
                        {#if index > 0}
                            <span
                                class="plus"
                                aria-hidden="true"
                                in:scale={keyPop}>+</span
                            >
                        {/if}
                        <kbd class="key" in:scale={keyPop}
                            >{partDisplayLabel(part)}</kbd
                        >
                    {/each}
                </div>

                {#if error}
                    <p class="advice" in:fade={{ duration: 120 }}>{error}</p>
                {/if}
            </div>
        </div>
    </div>
{/if}

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        z-index: 1000;
        background: transparent;
    }

    .popover-anchor {
        position: fixed;
        transform: translate(-50%, -100%);
        transform-origin: center bottom;
    }

    .popover {
        display: inline-flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        width: max-content;
        max-width: calc(100vw - 24px);
        padding: 10px 12px;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-sm);
        box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
        outline: none;
        transform-origin: center bottom;
    }

    .popover-error {
        border-color: var(--color-danger);
    }

    .combo {
        display: flex;
        flex-direction: row;
        flex-wrap: nowrap;
        align-items: center;
        justify-content: center;
        gap: 6px;
        min-height: 28px;
    }

    .plus {
        flex-shrink: 0;
        font-size: 12px;
        font-weight: 600;
        color: var(--color-text-tertiary);
        line-height: 1;
        user-select: none;
    }

    .key {
        flex-shrink: 0;
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
        font-family: inherit;
        color: var(--color-text-primary);
        letter-spacing: 0.02em;
        line-height: 1;
    }

    .popover-error .key {
        border-color: color-mix(
            in srgb,
            var(--color-danger) 55%,
            var(--color-border-medium)
        );
        color: var(--color-danger);
    }

    .advice {
        max-width: 220px;
        font-size: 11px;
        line-height: 1.4;
        text-align: center;
        color: var(--color-danger);
    }
</style>
