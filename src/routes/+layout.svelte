<script lang="ts">
    import { onMount } from "svelte";
    import "../global.css";

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
</style>
