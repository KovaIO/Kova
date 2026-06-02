<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

    type Phase = "idle" | "entering" | "settled";
    let phase: Phase = "idle";
    let generation = 0;

    function onOpened() {
        const id = ++generation;
        phase = "idle";
        queueMicrotask(() => {
            if (id !== generation) return;
            phase = "entering";
        });
    }

    function onClosed() {
        generation++;
        phase = "idle";
    }

    function onAnimationEnd(e: AnimationEvent) {
        if (e.animationName !== "popup") return;
        phase = "settled";
    }

    let unlistenFns: (() => void)[] = [];

    onMount(async () => {
        const webview = getCurrentWebviewWindow();

        unlistenFns.push(await webview.listen("window-opened", onOpened));
        unlistenFns.push(await webview.listen("window-closed", onClosed));

        if (await webview.isVisible()) {
            onOpened();
        }
    });

    onDestroy(() => {
        for (const unlisten of unlistenFns) unlisten();
    });
</script>

<div
    class="popup"
    class:entering={phase === "entering"}
    class:settled={phase === "settled"}
    onanimationend={onAnimationEnd}
>
    <slot />
</div>

<style>
    .popup {
        opacity: 0;
        transform: scale(0.5) translateY(8px);
    }

    .popup.entering {
        animation: popup 180ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
    }

    .popup.settled {
        opacity: 1;
        transform: scale(1) translateY(0);
    }

    @keyframes popup {
        from {
            opacity: 0;
            transform: scale(0.5) translateY(8px);
        }
        to {
            opacity: 1;
            transform: scale(1) translateY(0);
        }
    }
</style>
