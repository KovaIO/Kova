<script lang="ts">
    import { onMount } from "svelte";
    import "../global.css";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { loadPreferences, setPreferences } from "$stores/preferences";
    import { license, loadLicense } from "$stores/license";
    import type { LicenseInfo } from "$types/license";
    import type { Preferences } from "$types/preferences";

    onMount(async () => {
        await Promise.all([loadPreferences(), loadLicense()]);
    });

    onMount(() => {
        let unlisteners: UnlistenFn[] = [];

        (async () => {
            unlisteners = await Promise.all([
                listen<Preferences>("preferences-updated", (e) =>
                    setPreferences(e.payload),
                ),
                listen<LicenseInfo>("license-updated", (e) =>
                    license.set(e.payload),
                ),
            ]);
        })();

        return () => unlisteners.forEach((fn) => fn());
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
