<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { platform } from "@tauri-apps/plugin-os";
    import { onMount } from "svelte";
    import { ChevronLeft, ChevronRight } from "@lucide/svelte";
    import WindowAnimation from "$components/WindowAnimation.svelte";

    let current = $state(0);
    let isMacos = $state(false);
    let accessibilityGranted = $state(false);

    const allPages = [
        {
            title: "Kova",
            desc: "Your system, beautifully managed.",
            style: "welcome",
        },
        {
            title: "Accessibility",
            desc: "Kova needs accessibility access to enable keyboard shortcuts.",
            style: "accessibility",
        },
        {
            title: "Workspaces",
            desc: "Arrange your apps into workspace layouts.",
            style: "workspaces",
        },
        {
            title: "Clipboard History",
            desc: "Never lose what you've copied.",
            style: "clipboard",
        },
        {
            title: "System Monitor",
            desc: "Track CPU, RAM, and disk at a glance.",
            style: "monitor",
        },
        {
            title: "Preferences",
            desc: "Customize shortcuts and appearance.",
            style: "preferences",
        },
        { title: "You're all set!", desc: "Enjoy Kova.", style: "final" },
    ];

    let pages = $derived(
        isMacos
            ? allPages
            : allPages.filter((p) => p.style !== "accessibility"),
    );

    let page = $derived(pages[current]);

    onMount(async () => {
        const os = platform();
        isMacos = os === "macos";
        if (isMacos) {
            accessibilityGranted = await invoke("check_accessibility");
        }
    });

    function prev() {
        if (current > 0) current--;
    }

    function next() {
        if (current < pages.length - 1) current++;
    }

    async function finish() {
        await invoke("complete_onboarding");
        await getCurrentWindow().hide();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "ArrowLeft") prev();
        if (e.key === "ArrowRight") {
            if (current === pages.length - 1) finish();
            else next();
        }
        if (e.key === "Enter" && current === pages.length - 1) finish();
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<WindowAnimation>
    <div class="root">
        <div class="card">
            <button class="close-btn" onclick={finish} aria-label="Close"
                >✕</button
            >

            <div class="content">
                {#if current > 0}
                    <button
                        class="nav-arrow left"
                        onclick={prev}
                        aria-label="Previous"
                    >
                        <ChevronLeft size={18} />
                    </button>
                {/if}

                <div class="page">
                    {#if page.style === "welcome"}
                        <div class="illustration">
                            <img
                                src="/app-icon.png"
                                alt="Kova"
                                class="app-icon"
                            />
                        </div>
                    {:else if page.style === "accessibility"}
                        <div class="illustration">
                            <div class="a11y-window">
                                <div class="a11y-titlebar">
                                    <div class="a11y-dot"></div>
                                    <div class="a11y-dot"></div>
                                    <div class="a11y-dot"></div>
                                </div>
                                <div class="a11y-body">
                                    <div class="a11y-keyboard">
                                        {#each Array(3) as _, row}
                                            <div class="a11y-key-row">
                                                {#each Array(8 - row) as _}
                                                    <div class="a11y-key"></div>
                                                {/each}
                                            </div>
                                        {/each}
                                        <div class="a11y-spacebar"></div>
                                    </div>
                                </div>
                            </div>
                            {#if accessibilityGranted}
                                <div class="a11y-granted">
                                    <div class="a11y-check">✓</div>
                                    <span>Granted</span>
                                </div>
                            {/if}
                        </div>
                    {:else if page.style === "workspaces"}
                        <div class="illustration">
                            <div class="ws-window">
                                <div class="ws-titlebar">
                                    <div class="ws-dot"></div>
                                    <div class="ws-dot"></div>
                                    <div class="ws-dot"></div>
                                </div>
                                <div class="ws-body">
                                    <div class="ws-ring">
                                        {#each [0, 1, 2, 3, 4, 5] as _, i}
                                            {@const angle = (360 / 6) * i - 90}
                                            {@const rad =
                                                (angle * Math.PI) / 180}
                                            {@const r = 72}
                                            {@const x = Math.cos(rad) * r}
                                            {@const y = Math.sin(rad) * r}
                                            <div
                                                class="ws-profile"
                                                class:ws-active={i === 0}
                                                style="transform: translate({x}px, {y}px)"
                                            >
                                                <div class="ws-grid">
                                                    <div
                                                        class="ws-slot ws-s1"
                                                    ></div>
                                                    <div
                                                        class="ws-slot ws-s2"
                                                    ></div>
                                                    <div
                                                        class="ws-slot ws-s3"
                                                    ></div>
                                                </div>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            </div>
                        </div>
                    {:else if page.style === "clipboard"}
                        <div class="illustration">
                            <div class="cb-clip">
                                <div class="cb-clip-head"></div>
                            </div>
                            <div class="cb-board">
                                <div class="cb-item cb-i1"></div>
                                <div class="cb-item cb-i2"></div>
                                <div class="cb-item cb-i3"></div>
                                <div class="cb-item cb-i4"></div>
                            </div>
                        </div>
                    {:else if page.style === "monitor"}
                        <div class="illustration">
                            <div class="mon-window">
                                <div class="mon-tabs">
                                    <div class="mon-tab mon-active">CPU</div>
                                    <div class="mon-tab">Memory</div>
                                    <div class="mon-tab">Network</div>
                                </div>
                                <div class="mon-chart">
                                    <div class="mon-grid">
                                        <div class="mon-grid-line"></div>
                                        <div class="mon-grid-line"></div>
                                        <div class="mon-grid-line"></div>
                                        <div class="mon-grid-line"></div>
                                    </div>
                                    <div class="mon-bars">
                                        {#each [35, 52, 45, 68, 42, 55, 73, 60, 48, 80, 65, 50, 70, 58, 44, 75, 62, 53, 67, 56] as h, i}
                                            <div
                                                class="mon-bar"
                                                style="height: {h}%; animation-delay: {i *
                                                    60}ms"
                                            >
                                                <div class="mon-bar-top"></div>
                                                <div
                                                    class="mon-bar-bottom"
                                                ></div>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            </div>
                        </div>
                    {:else if page.style === "preferences"}
                        <div class="illustration">
                            <div class="pref-window">
                                <div class="pref-titlebar">
                                    <div class="pref-dot"></div>
                                    <div class="pref-dot"></div>
                                    <div class="pref-dot"></div>
                                </div>
                                <div class="pref-body">
                                    <div class="pref-sidebar">
                                        <div
                                            class="pref-nav-item pref-nav-active"
                                        ></div>
                                        <div class="pref-nav-item"></div>
                                        <div class="pref-nav-item"></div>
                                        <div class="pref-nav-item"></div>
                                        <div class="pref-nav-item"></div>
                                    </div>
                                    <div class="pref-content">
                                        <div
                                            class="pref-line pref-line-w60"
                                        ></div>
                                        <div class="pref-spacer"></div>
                                        <div class="pref-row">
                                            <div
                                                class="pref-line pref-line-w40"
                                            ></div>
                                            <div class="pref-toggle"></div>
                                        </div>
                                        <div class="pref-row">
                                            <div
                                                class="pref-line pref-line-w50"
                                            ></div>
                                            <div class="pref-toggle"></div>
                                        </div>
                                        <div class="pref-row">
                                            <div
                                                class="pref-line pref-line-w35"
                                            ></div>
                                            <div class="pref-toggle"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    {:else if page.style === "final"}
                        <div class="illustration final-illustration">
                            <div class="final-sparkle s1">✦</div>
                            <div class="final-sparkle s2">✦</div>
                            <div class="final-sparkle s3">✦</div>
                            <div class="final-check">✓</div>
                        </div>
                    {/if}

                    <h1 class="title">{page.title}</h1>
                    <p class="desc">{page.desc}</p>

                    {#if page.style === "final"}
                        <button class="btn-primary" onclick={finish}
                            >Let's go</button
                        >
                    {/if}
                </div>

                {#if current < pages.length - 1}
                    <button
                        class="nav-arrow right"
                        onclick={next}
                        aria-label="Next"
                    >
                        <ChevronRight size={18} />
                    </button>
                {/if}
            </div>

            <div class="dots">
                {#each pages as _, i}
                    <button
                        class="dot"
                        class:active={i === current}
                        class:past={i < current}
                        onclick={() => (current = i)}
                        aria-label="Go to page {i + 1}"
                    ></button>
                {/each}
            </div>
        </div>
    </div>
</WindowAnimation>

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
        position: relative;
        width: 560px;
        height: 480px;
        background: var(--color-main-bg);
        border: 2px solid var(--color-border-subtle);
        border-radius: var(--radius-lg);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        user-select: none;
    }

    .close-btn {
        position: absolute;
        top: 14px;
        left: 14px;
        width: 26px;
        height: 26px;
        border-radius: 50%;
        border: none;
        background: var(--color-button-bg);
        color: var(--color-text-dim);
        font-size: 11px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: var(--transition-fast);
        z-index: 2;
    }

    .close-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-secondary);
    }

    .content {
        flex: 1;
        display: flex;
        align-items: center;
        position: relative;
        min-height: 0;
    }

    .page {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 10px;
        padding: 30px 50px;
        text-align: center;
    }

    /* ── Nav arrows ── */
    .nav-arrow {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        background: none;
        border: none;
        color: var(--color-text-dim);
        cursor: pointer;
        padding: 8px;
        z-index: 2;
        transition: var(--transition-fast);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .nav-arrow:hover {
        color: var(--color-text-secondary);
    }

    .nav-arrow.left {
        left: 8px;
    }
    .nav-arrow.right {
        right: 8px;
    }

    /* ── Dots ── */
    .dots {
        display: flex;
        gap: 8px;
        justify-content: center;
        padding: 14px 0 18px;
    }

    .dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        border: none;
        background: var(--color-button-bg-hover);
        cursor: pointer;
        padding: 0;
        transition: all 260ms cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .dot.past {
        background: var(--color-text-dim);
    }

    .dot.active {
        background: var(--color-accent);
        width: 18px;
        border-radius: 4px;
    }

    .dot:hover:not(.active) {
        background: var(--color-text-dim);
    }

    /* ── Typography ── */
    .title {
        font-size: 20px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin: 0;
        line-height: 1.2;
    }

    .desc {
        font-size: 13px;
        color: var(--color-text-tertiary);
        margin: 0;
        max-width: 320px;
        line-height: 1.5;
    }

    .illustration {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 12px;
        position: relative;
        height: 200px;
    }

    .btn-primary {
        margin-top: 12px;
        padding: 10px 28px;
        border-radius: 10px;
        border: none;
        background: var(--color-accent);
        color: #fff;
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        cursor: pointer;
        transition: var(--transition-fast);
    }

    .btn-primary:hover {
        background: var(--color-accent-hover);
    }

    /* ═══════════════════════════════════════════════
       WELCOME
       ═══════════════════════════════════════════════ */
    .app-icon {
        width: 80px;
        height: 80px;
        border-radius: 18px;
    }

    /* ═══════════════════════════════════════════════
       ACCESSIBILITY
       ═══════════════════════════════════════════════ */
    .a11y-window {
        width: 220px;
        background: var(--color-surface-elevated);
        border-radius: 10px;
        border: 1px solid var(--color-border-medium);
        overflow: hidden;
    }

    .a11y-titlebar {
        display: flex;
        gap: 5px;
        padding: 8px 10px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .a11y-dot {
        width: 7px;
        height: 7px;
        border-radius: 50%;
        background: var(--color-button-bg-hover);
    }

    .a11y-body {
        padding: 16px;
        display: flex;
        justify-content: center;
    }

    .a11y-keyboard {
        display: flex;
        flex-direction: column;
        gap: 4px;
        align-items: center;
    }

    .a11y-key-row {
        display: flex;
        gap: 3px;
    }

    .a11y-key {
        width: 18px;
        height: 14px;
        border-radius: 3px;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-medium);
    }

    .a11y-spacebar {
        width: 80px;
        height: 12px;
        border-radius: 3px;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-medium);
    }

    .a11y-granted {
        position: absolute;
        bottom: 0;
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 6px 14px;
        border-radius: 20px;
        background: var(--color-success-soft);
        color: var(--color-success);
        font-size: 12px;
        font-weight: 500;
    }

    .a11y-check {
        font-weight: 700;
    }

    /* ═══════════════════════════════════════════════
       WORKSPACES
       ═══════════════════════════════════════════════ */
    .ws-window {
        width: 240px;
        height: 160px;
        background: var(--color-surface-elevated);
        border-radius: 10px;
        border: 1px solid var(--color-border-medium);
        overflow: hidden;
    }

    .ws-titlebar {
        display: flex;
        gap: 5px;
        padding: 7px 10px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .ws-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--color-button-bg-hover);
    }

    .ws-body {
        display: flex;
        align-items: center;
        justify-content: center;
        height: calc(100% - 28px);
    }

    .ws-ring {
        position: relative;
        width: 180px;
        height: 130px;
    }

    .ws-profile {
        position: absolute;
        top: 50%;
        left: 50%;
        width: 52px;
        height: 36px;
        margin: -18px 0 0 -26px;
        background: var(--color-main-bg);
        border-radius: 5px;
        border: 1px solid var(--color-border-strong);
        overflow: hidden;
        transition: all 360ms cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .ws-profile.ws-active {
        border-color: var(--color-accent);
        transform-origin: center;
        z-index: 1;
    }

    .ws-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        grid-template-rows: 1fr 1fr;
        gap: 2px;
        padding: 3px;
        height: 100%;
    }

    .ws-slot {
        background: rgba(255, 255, 255, 0.08);
        border-radius: 2px;
    }

    .ws-s1 {
        grid-column: 1 / 3;
    }
    .ws-s2 {
        grid-row: 2;
    }
    .ws-s3 {
        grid-row: 2;
    }

    /* ═══════════════════════════════════════════════
       CLIPBOARD
       ═══════════════════════════════════════════════ */
    .cb-clip {
        position: relative;
        display: flex;
        justify-content: center;
        z-index: 2;
    }

    .cb-clip-head {
        width: 48px;
        height: 18px;
        border: 3px solid var(--color-text-dim);
        border-bottom: none;
        border-radius: 8px 8px 0 0;
        position: relative;
        top: 2px;
    }

    .cb-board {
        position: relative;
        width: 160px;
        height: 140px;
        background: var(--color-surface-elevated);
        border-radius: 10px;
        border: 1px solid var(--color-border-medium);
        padding: 14px 12px;
        display: flex;
        flex-direction: column;
        gap: 6px;
        z-index: 1;
        margin-top: -8px;
    }

    .cb-item {
        height: 22px;
        border-radius: 5px;
        background: var(--color-button-bg);
    }

    .cb-i1 {
        width: 90%;
    }
    .cb-i2 {
        width: 75%;
    }
    .cb-i3 {
        width: 85%;
    }
    .cb-i4 {
        width: 60%;
    }

    /* ═══════════════════════════════════════════════
       SYSTEM MONITOR
       ═══════════════════════════════════════════════ */
    .mon-window {
        width: 260px;
        background: var(--color-surface-elevated);
        border-radius: 10px;
        border: 1px solid var(--color-border-medium);
        overflow: hidden;
    }

    .mon-tabs {
        display: flex;
        gap: 4px;
        padding: 8px 10px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .mon-tab {
        padding: 3px 10px;
        border-radius: 999px;
        font-size: 9px;
        font-weight: 500;
        color: var(--color-text-dim);
        background: transparent;
    }

    .mon-active {
        background: var(--color-accent-soft);
        color: var(--color-accent);
    }

    .mon-chart {
        position: relative;
        height: 120px;
        padding: 8px 10px;
    }

    .mon-grid {
        position: absolute;
        inset: 8px 10px;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
    }

    .mon-grid-line {
        height: 1px;
        background: var(--color-border-subtle);
    }

    .mon-bars {
        position: absolute;
        inset: 8px 10px;
        display: flex;
        align-items: flex-end;
        gap: 2px;
    }

    .mon-bar {
        flex: 1;
        display: flex;
        flex-direction: column;
        border-radius: 2px 2px 0 0;
        overflow: hidden;
        animation: mon-grow 800ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    }

    @keyframes mon-grow {
        from {
            height: 0 !important;
        }
    }

    .mon-bar-top {
        background: var(--color-accent);
        flex-shrink: 0;
        height: 30%;
    }

    .mon-bar-bottom {
        background: var(--color-accent-soft);
        flex: 1;
    }

    /* ═══════════════════════════════════════════════
       PREFERENCES
       ═══════════════════════════════════════════════ */
    .pref-window {
        width: 260px;
        height: 160px;
        background: var(--color-surface-elevated);
        border-radius: 10px;
        border: 1px solid var(--color-border-medium);
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .pref-titlebar {
        display: flex;
        gap: 5px;
        padding: 7px 10px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .pref-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--color-button-bg-hover);
    }

    .pref-body {
        display: flex;
        flex: 1;
        min-height: 0;
    }

    .pref-sidebar {
        width: 70px;
        border-right: 1px solid var(--color-border-subtle);
        padding: 8px 6px;
        display: flex;
        flex-direction: column;
        gap: 3px;
    }

    .pref-nav-item {
        height: 14px;
        border-radius: 4px;
        background: var(--color-button-bg);
    }

    .pref-nav-active {
        background: var(--color-accent-soft);
    }

    .pref-content {
        flex: 1;
        padding: 12px;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .pref-line {
        height: 8px;
        border-radius: 4px;
        background: var(--color-button-bg);
    }

    .pref-line-w35 {
        width: 35%;
    }
    .pref-line-w40 {
        width: 40%;
    }
    .pref-line-w50 {
        width: 50%;
    }
    .pref-line-w60 {
        width: 60%;
    }

    .pref-spacer {
        height: 4px;
    }

    .pref-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
    }

    .pref-toggle {
        width: 22px;
        height: 12px;
        border-radius: 6px;
        background: var(--color-track-fill);
        flex-shrink: 0;
    }

    /* ═══════════════════════════════════════════════
       FINAL
       ═══════════════════════════════════════════════ */
    .final-illustration {
        align-items: center;
    }

    .final-check {
        width: 64px;
        height: 64px;
        border-radius: 50%;
        background: var(--color-success-soft);
        color: var(--color-success);
        font-size: 28px;
        font-weight: 700;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .final-sparkle {
        position: absolute;
        color: var(--color-accent);
        font-size: 14px;
        opacity: 0.5;
        animation: sparkle-pulse 2s ease-in-out infinite;
    }

    .s1 {
        top: 30px;
        right: 60px;
        animation-delay: 0s;
    }
    .s2 {
        top: 60px;
        left: 55px;
        animation-delay: 0.7s;
        font-size: 10px;
    }
    .s3 {
        bottom: 40px;
        right: 80px;
        animation-delay: 1.4s;
        font-size: 11px;
    }

    @keyframes sparkle-pulse {
        0%,
        100% {
            opacity: 0.3;
            transform: scale(1);
        }
        50% {
            opacity: 0.8;
            transform: scale(1.3);
        }
    }
</style>
