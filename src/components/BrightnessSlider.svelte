<script lang="ts">
    import { Sun } from "@lucide/svelte";

    export let value: number;
    export let disabled = false;

    export let onchange: (value: number) => void = () => {};

    let localValue = value;
    let dragging = false;

    $: if (!dragging) {
        localValue = value;
    }

    function onInput(e: Event) {
        localValue = +(e.target as HTMLInputElement).value;
    }

    function onPointerDown() {
        if (disabled) return;
        dragging = true;
    }

    function onPointerUp() {
        if (disabled) return;
        dragging = false;
        onchange(localValue);
    }
</script>

<div class="brightness-row" class:disabled>
    <Sun class="icon" size={14} />
    <div class="track-wrap" style="--thumb-position: {localValue}%">
        <input
            type="range"
            min="0"
            max="100"
            value={localValue}
            {disabled}
            on:input={onInput}
            on:pointerdown={onPointerDown}
            on:pointerup={onPointerUp}
            aria-label="Brightness"
        />
        <div class="track-fill" style="width: {localValue}%"></div>
    </div>
</div>

<style>
    .brightness-row {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .brightness-row.disabled {
        opacity: 0.45;
        pointer-events: none;
    }

    .track-wrap {
        position: relative;
        flex: 1;
        height: 4px;
    }

    input[type="range"] {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        opacity: 0;
        z-index: 2;
        margin: 0;
        height: 20px;
        top: 50%;
        transform: translateY(-50%);
    }

    .track-fill {
        position: absolute;
        top: 0;
        left: 0;
        bottom: 0;
        background: linear-gradient(
            to right,
            var(--color-accent),
            var(--color-accent-hover)
        );
        border-radius: 999px;
        pointer-events: none;
        transition: width var(--transition-fast);
        opacity: 0.9;
    }

    .track-wrap::before {
        content: "";
        position: absolute;
        inset: 0;
        background: var(--color-track-base);
        border-radius: 999px;
        border: 1px solid var(--color-border-subtle);
    }

    .track-wrap::after {
        content: "";
        position: absolute;
        top: 50%;
        left: calc(var(--thumb-position) - 6px);
        transform: translateY(-50%);
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: var(--color-input);
        pointer-events: none;
        transition: left var(--transition-fast);
        z-index: 1;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }
</style>
