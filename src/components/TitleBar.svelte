<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { platform } from "@tauri-apps/plugin-os";
    import { X } from "@lucide/svelte";

    const isMac = platform() === "macos";
    let hovered = false;

    async function close() {
        const win = getCurrentWindow();
        await win.hide();
        hovered = false;
    }
</script>

<div class="titlebar">
    <div class="slot">
        {#if isMac}
            <button
                class="close-btn"
                class:hovered
                on:click={close}
                on:mouseenter={() => (hovered = true)}
                on:mouseleave={() => (hovered = false)}
                aria-label="Close"
            >
                <X size={14} />
            </button>
        {/if}
    </div>

    <div class="slot right">
        {#if !isMac}
            <button
                class="close-btn"
                class:hovered
                on:click={close}
                on:mouseenter={() => (hovered = true)}
                on:mouseleave={() => (hovered = false)}
                aria-label="Close"
            >
                <X size={14} />
            </button>
        {/if}
    </div>
</div>

<style>
    .titlebar {
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 14px 0 18px;
        flex-shrink: 0;
        user-select: none;
        -webkit-app-region: drag;
    }

    .slot {
        width: 26px;
        display: flex;
        align-items: center;
        -webkit-app-region: no-drag;
    }

    .right {
        justify-content: flex-end;
    }

    .close-btn {
        width: 26px;
        height: 26px;
        border-radius: 50%;
        border: none;
        background: var(--color-track-bg);
        color: var(--color-text-dim);
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        transition: var(--transition-fast);
        -webkit-app-region: no-drag;
    }

    .close-btn.hovered {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }
</style>
