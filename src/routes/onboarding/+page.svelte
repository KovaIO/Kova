<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import WindowAnimation from "$components/WindowAnimation.svelte";

    async function complete() {
        await invoke("complete_onboarding");
        await getCurrentWindow().hide();
    }
</script>

<WindowAnimation>
    <div class="root">
        <h1>Welcome to Kova</h1>
        <p class="subtitle">Your system, beautifully managed.</p>

        <button class="btn-primary" on:click={complete}>Get Started</button>
    </div>
</WindowAnimation>

<style>
    .root {
        width: 100vw;
        height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--color-main-bg);
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        padding: 10px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        text-align: center;
    }

    h1 {
        font-size: 20px;
        font-weight: 600;
        color: var(--color-text-primary, #fff);
        margin: 0;
    }

    .subtitle {
        font-size: 13px;
        color: var(--color-text-tertiary);
        margin: 0 0 24px 0;
    }

    .btn-primary {
        padding: 10px 24px;
        border-radius: 10px;
        border: none;
        background: var(--color-accent);
        color: #fff;
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        cursor: pointer;
        transition: background 150ms ease;
    }

    .btn-primary:hover {
        background: var(--color-accent-hover);
    }
</style>
