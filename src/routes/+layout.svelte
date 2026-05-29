<script lang="ts">
    import { onMount } from "svelte";
    import "../global.css";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { loadPreferences, preferences } from "$stores/preferences";
    import type { Preferences } from "$types/preferences";

    onMount(() => {
        let unlisten: UnlistenFn | undefined;

        (async () => {
            unlisten = await listen<Preferences>(
                "preferences-updated",
                (event) => {
                    preferences.set(event.payload);
                },
            );
        })();

        return () => unlisten?.();
    });

    onMount(async () => {
        await loadPreferences();
    });

    onMount(() => {
        const disableContextMenu = (e: MouseEvent) => {
            e.preventDefault();
        };

        document.addEventListener("contextmenu", disableContextMenu);

        return () => {
            document.removeEventListener("contextmenu", disableContextMenu);
        };
    });
</script>

<slot />

<style>
    :global(html, body) {
        width: 100%;
        margin: 0;
        padding: 0;
        background: transparent;
        overflow: hidden;
    }

    :global(*, *::before, *::after) {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    :global(body) {
        font-family:
            "SF Pro Display",
            -apple-system,
            "Segoe UI Variable",
            "Segoe UI",
            sans-serif;
        -webkit-font-smoothing: antialiased;
    }
</style>
