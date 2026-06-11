<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import WindowAnimation from "$components/WindowAnimation.svelte";

    let version = "";
    let notes = "";
    let installing = false;

    onMount(async () => {
        const update = await invoke<{ version: string; notes: string } | null>(
            "get_update",
        );
        if (update) {
            version = update.version;
            notes = update.notes;
        }
    });

    async function install() {
        if (installing) return;
        installing = true;
        try {
            await invoke("install_update");
        } catch (e) {
            installing = false;
        }
    }

    async function dismiss() {
        await invoke("dismiss_update");
        await getCurrentWindow().hide();
    }
</script>

<div class="root">
    <div class="card">
        <div class="header">
            <h1>Update Available</h1>
            <p class="version">Kova v{version}</p>
        </div>

        {#if notes}
            <div class="notes">
                <h2>What's New</h2>
                <p>{notes}</p>
            </div>
        {/if}

        <div class="actions">
            <button class="btn-secondary" on:click={dismiss}>Later</button>
            <button
                class="btn-primary"
                on:click={install}
                disabled={installing}
            >
                {installing ? "Installing..." : "Install Now"}
            </button>
        </div>
    </div>
</div>

<style>
    .root {
        width: 100vw;
        height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        background: transparent;
    }

    .card {
        width: 360px;
        background: var(--color-main-bg, #1a1a1a);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 24px;
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .header {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    h1 {
        font-size: 15px;
        font-weight: 600;
        color: var(--color-text-primary, #fff);
        margin: 0;
    }

    .version {
        font-size: 12px;
        color: var(--color-text-tertiary, rgba(255, 255, 255, 0.4));
        margin: 0;
    }

    .notes {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 12px;
        max-height: 120px;
        overflow-y: auto;
    }

    .notes h2 {
        font-size: 11px;
        font-weight: 600;
        color: var(--color-text-secondary, rgba(255, 255, 255, 0.6));
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin: 0 0 6px 0;
    }

    .notes p {
        font-size: 12px;
        line-height: 1.5;
        color: var(--color-text-primary, rgba(255, 255, 255, 0.85));
        white-space: pre-wrap;
        margin: 0;
    }

    .actions {
        display: flex;
        gap: 8px;
        justify-content: flex-end;
    }

    .btn-secondary,
    .btn-primary {
        padding: 8px 16px;
        border-radius: 6px;
        border: none;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition:
            background 150ms ease,
            opacity 150ms ease;
    }

    .btn-secondary {
        background: rgba(255, 255, 255, 0.08);
        color: var(--color-text-primary, rgba(255, 255, 255, 0.85));
    }

    .btn-secondary:hover {
        background: rgba(255, 255, 255, 0.12);
    }

    .btn-primary {
        background: var(--color-accent, #6366f1);
        color: #fff;
    }

    .btn-primary:hover {
        background: var(--color-accent-hover, #5558e6);
    }

    .btn-primary:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
