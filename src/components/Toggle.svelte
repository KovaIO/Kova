<script lang="ts">
    export let label: string;
    export let checked: boolean = false;
    export let id: string;
</script>

<label class="toggle-row" for={id}>
    <span class="toggle-label">{label}</span>
    <div class="switch">
        <input type="checkbox" {id} bind:checked />
        <span class="track">
            <span class="thumb"></span>
        </span>
    </div>
</label>

<style>
    .toggle-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        user-select: none;
        padding: 2px 0;
    }

    .toggle-label {
        font-size: 12.5px;
        font-weight: 450;
        color: var(--color-text-secondary);
        letter-spacing: 0.01em;
    }

    input {
        display: none;
    }

    .switch {
        position: relative;
        width: 36px;
        height: 20px;
        flex-shrink: 0;
    }

    .track {
        position: absolute;
        inset: 0;
        border-radius: 999px;
        background: var(--color-track-bg);
        border: 1px solid var(--color-border-medium);
        transition: var(--transition-medium);
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
    }

    input:checked ~ .track {
        background: var(--color-accent);
        border-color: var(--color-accent-border);
    }

    .thumb {
        position: absolute;
        top: 3px;
        left: 3px;
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: var(--color-input);
        transition: var(--transition-medium);
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }

    input:checked ~ .track .thumb {
        transform: translateX(16px);
        background: var(--color-input-active);
    }
</style>
