<script lang="ts">
    export let value: number = 80;
</script>

<div class="brightness-row">
    <svg
        class="icon"
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
    >
        <circle cx="12" cy="12" r="4" />
        <line x1="12" y1="2" x2="12" y2="4" />
        <line x1="12" y1="20" x2="12" y2="22" />
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
        <line x1="2" y1="12" x2="4" y2="12" />
        <line x1="20" y1="12" x2="22" y2="12" />
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
    </svg>
    <div class="track-wrap" style="--thumb-position: {value}%">
        <input
            type="range"
            min="0"
            max="100"
            bind:value
            aria-label="Brightness"
        />
        <div class="track-fill" style="width: {value}%"></div>
    </div>
</div>

<style>
    .brightness-row {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .icon {
        color: var(--color-text-dim);
        flex-shrink: 0;
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
        cursor: pointer;
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
        background: linear-gradient(to right, var(--color-accent), var(--color-accent-hover));
        border-radius: 999px;
        pointer-events: none;
        transition: width 60ms linear;
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
        transition: left 60ms linear;
        z-index: 1;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }
</style>
