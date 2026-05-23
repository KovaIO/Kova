<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import Titlebar from "$components/TitleBar.svelte";

    onMount(() => {
        const win = getCurrentWindow();
        const unlisten = win.onCloseRequested((event) => {
            event.preventDefault();
            win.hide();
        });

        return () => {
            unlisten.then((f) => f());
        };
    });
</script>

<div class="window">
    <Titlebar />
    <div class="content"></div>
</div>

<style>
    .window {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border-radius: var(--radius-md);
        border: 2px solid var(--color-border-subtle);
        overflow: hidden;
        color: white;
    }

    .content {
        flex: 1;
        overflow-y: auto;
        padding: 20px 24px;
    }
</style>
