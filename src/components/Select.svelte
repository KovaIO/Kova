<script lang="ts">
    import { fly } from "svelte/transition";
    import { ChevronDown } from "@lucide/svelte";

    let {
        value = $bindable(),
        options,
        onchange,
    }: {
        value: string | number;
        options: { value: string | number; label: string; disabled?: boolean }[];
        onchange?: (value: string | number) => void;
    } = $props();

    let open = $state(false);
    let container: HTMLDivElement;

    let selected = $derived(options.find((o) => o.value === value));

    function select(opt: (typeof options)[number]) {
        if (opt.disabled) return;
        value = opt.value;
        open = false;
        onchange?.(opt.value);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            open = false;
        } else if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            open = !open;
        } else if (e.key === "ArrowDown" && open) {
            e.preventDefault();
            const enabled = options.filter((o) => !o.disabled);
            const idx = enabled.findIndex((o) => o.value === value);
            const next = enabled[(idx + 1) % enabled.length];
            select(next);
        } else if (e.key === "ArrowUp" && open) {
            e.preventDefault();
            const enabled = options.filter((o) => !o.disabled);
            const idx = enabled.findIndex((o) => o.value === value);
            const prev = enabled[(idx - 1 + enabled.length) % enabled.length];
            select(prev);
        }
    }

    function handleClickOutside(e: MouseEvent) {
        if (container && !container.contains(e.target as Node)) {
            open = false;
        }
    }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="select-container" bind:this={container}>
    <button
        class="select-trigger"
        class:open
        type="button"
        onclick={() => (open = !open)}
        onkeydown={handleKeydown}
        aria-haspopup="listbox"
        aria-expanded={open}
    >
        <span class="select-label">{selected?.label ?? "Select…"}</span>
        <span class="chevron-wrap">
            <ChevronDown size={14} class="chevron" />
        </span>
    </button>

    {#if open}
        <div
            class="select-dropdown"
            role="listbox"
            transition:fly={{ y: -4, duration: 150 }}
        >
            {#each options as opt}
                <button
                    class="select-option"
                    class:selected={opt.value === value}
                    class:disabled={opt.disabled}
                    role="option"
                    aria-selected={opt.value === value}
                    type="button"
                    onclick={() => select(opt)}
                >
                    <span class="option-label">{opt.label}</span>
                </button>
            {/each}
        </div>
    {/if}
</div>

<style>
    .select-container {
        position: relative;
        display: inline-flex;
    }

    .select-trigger {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 7px 10px 7px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-primary);
        font-size: 13px;
        font-family: inherit;
        cursor: pointer;
        transition:
            background var(--transition-fast),
            border-color var(--transition-fast);
        min-width: 120px;
        justify-content: space-between;
    }

    .select-trigger:hover {
        background: var(--color-button-bg-hover);
    }

    .select-trigger.open {
        border-color: var(--color-accent-border);
    }

    .select-trigger :global(.chevron) {
        color: var(--color-text-muted);
        flex-shrink: 0;
    }

    .chevron-wrap {
        display: flex;
        align-items: center;
        transition: transform var(--transition-fast);
    }

    .select-trigger.open .chevron-wrap {
        transform: rotate(180deg);
    }

    .select-label {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .select-dropdown {
        position: absolute;
        top: calc(100% + 4px);
        left: 0;
        right: 0;
        min-width: 100%;
        z-index: 50;
        padding: 4px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-surface-elevated);
        box-shadow:
            0 8px 24px rgba(0, 0, 0, 0.35),
            0 2px 6px rgba(0, 0, 0, 0.2);
    }

    .select-option {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        width: 100%;
        padding: 7px 10px 7px 14px;
        border: none;
        background: transparent;
        color: var(--color-text-secondary);
        font-size: 13px;
        font-family: inherit;
        cursor: pointer;
        transition:
            color var(--transition-fast);
        text-align: left;
        position: relative;
    }

    .select-option:hover {
        color: var(--color-text-primary);
    }

    .select-option.selected {
        color: var(--color-accent);
        font-weight: 500;
    }

    .select-option.selected::before {
        content: "";
        position: absolute;
        left: 0;
        top: 6px;
        bottom: 6px;
        width: 3px;
        border-radius: 0 3px 3px 0;
        background: var(--color-accent);
    }

    .select-option.disabled {
        opacity: 0.35;
        cursor: not-allowed;
    }

    .select-option.disabled:hover {
        color: var(--color-text-secondary);
    }

    .option-label {
        flex: 1;
    }
</style>
