<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import {
        HardDrive,
        Clipboard,
        LayoutGrid,
        Settings,
        Cpu,
        ChevronLeft,
        ChevronRight,
    } from "@lucide/svelte";
    import WindowAnimation from "$components/WindowAnimation.svelte";

    let current = $state(0);

    const pages = [
        { style: "welcome" },
        { style: "workspaces" },
        { style: "clipboard" },
        { style: "monitor" },
        { style: "preferences" },
        { style: "final" },
    ];

    let page = $derived(pages[current]);

    function goNext() {
        if (current < pages.length - 1) current++;
    }

    function goPrev() {
        if (current > 0) current--;
    }

    async function finish() {
        await invoke("complete_onboarding");
        await getCurrentWindow().hide();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "ArrowLeft") goPrev();
        if (e.key === "ArrowRight") goNext();
        if (e.key === "Enter" && current === pages.length - 1) finish();
    }

    const pageContent: Record<string, { title: string; desc: string }> = {
        welcome: {
            title: "Meet Kova",
            desc: "Your productivity companion.",
        },
        workspaces: {
            title: "Workspaces",
            desc: "Save your perfect app layout and restore it with one shortcut.",
        },
        clipboard: {
            title: "Clipboard History",
            desc: "Never lose what you copied. Search, paste, and manage everything.",
        },
        monitor: {
            title: "System Monitor",
            desc: "Keep an eye on CPU, memory, and network in real time.",
        },
        preferences: {
            title: "Make It Yours",
            desc: "Themes, shortcuts, and settings, all configurable.",
        },
        final: {
            title: "All Set",
            desc: "Enjoy Kova, we are grateful to have you on board",
        },
    };

    const fullbleedStyles = ["workspaces", "clipboard", "monitor", "preferences"];
</script>

<svelte:window on:keydown={handleKeydown} />

<WindowAnimation>
    <div class="root">
        <div class="card">
            <!-- Close button -->
            <button class="close-btn" onclick={finish} aria-label="Close">
                ✕
            </button>

            <!-- Content (integrated illustration + text) -->
            <div class="content">
                <div
                    class="illustration"
                    class:fullbleed={fullbleedStyles.includes(page.style)}
                >
                    {#if page.style === "welcome"}
                        <div class="welcome-scene">
                            <img
                                src="/app-icon.png"
                                alt="Kova"
                                class="app-icon"
                            />
                        </div>

                    {:else if page.style === "workspaces"}
                        <div class="workspace-layout">
                            <div class="ws-main">
                                <span class="ws-title">{pageContent.workspaces.title}</span>
                                <span class="ws-desc">{pageContent.workspaces.desc}</span>
                            </div>
                            <div class="ws-side">
                                <div class="ws-top"></div>
                                <div class="ws-bottom"></div>
                            </div>
                        </div>

                    {:else if page.style === "clipboard"}
                        <div class="clipboard-visual">
                            <div class="clip-clipboard">
                                <div class="clip-clip"></div>
                                <div class="clip-body">
                                    <span class="clip-title">{pageContent.clipboard.title}</span>
                                    <span class="clip-desc">{pageContent.clipboard.desc}</span>
                                </div>
                            </div>
                        </div>

                    {:else if page.style === "monitor"}
                        <div class="monitor-vis">
                            <div class="monitor-text">
                                <span class="mon-title">{pageContent.monitor.title}</span>
                                <span class="mon-desc">{pageContent.monitor.desc}</span>
                            </div>
                            <div class="vis-bars">
                                {#each [35, 55, 45, 75, 50, 65, 40, 70, 55, 80, 45, 65, 50, 70, 60, 35, 55, 45, 75, 50, 60, 40, 75, 50, 45, 65, 55, 80, 40, 70, 50, 65, 45, 75, 55, 60, 40, 80, 50, 70] as h, i}
                                    <div
                                        class="vis-bar"
                                        style="height: {h}%; animation-delay: {i *
                                            30}ms"
                                    >
                                        <div class="bar-top"></div>
                                        <div class="bar-bottom"></div>
                                    </div>
                                {/each}
                            </div>
                        </div>

                    {:else if page.style === "preferences"}
                        <div class="prefs-gears">
                            <div class="gear gear-tr">
                                <Settings size={140} strokeWidth={1} />
                            </div>
                            <div class="gear gear-bl">
                                <Settings size={120} strokeWidth={1} />
                            </div>
                            <div class="prefs-text">
                                <span class="prefs-title">{pageContent.preferences.title}</span>
                                <span class="prefs-desc">{pageContent.preferences.desc}</span>
                            </div>
                        </div>

                    {:else if page.style === "final"}
                        <div class="final-check">
                            <div class="check-icon">✓</div>
                        </div>
                    {/if}
                </div>

                {#if !fullbleedStyles.includes(page.style)}
                    <div class="text">
                        <h1 class="title">{pageContent[page.style].title}</h1>
                        <p class="desc">{pageContent[page.style].desc}</p>
                    </div>
                {/if}
            </div>

            <!-- Dots (centered) -->
            <div class="dots">
                {#each pages as _, i}
                    <button
                        class="dot"
                        class:active={i === current}
                        class:past={i < current}
                        onclick={() => (current = i)}
                        aria-label="Step {i + 1}"
                    ></button>
                {/each}
            </div>

            <!-- Navigation arrows -->
            {#if current > 0}
                <button
                    class="nav-arrow left"
                    onclick={goPrev}
                    aria-label="Previous"
                >
                    <ChevronLeft size={22} />
                </button>
            {/if}
            {#if current < pages.length - 1}
                <button
                    class="nav-arrow right"
                    onclick={goNext}
                    aria-label="Next"
                >
                    <ChevronRight size={22} />
                </button>
            {/if}

            <!-- Final button (only on last page) -->
            {#if current === pages.length - 1}
                <button class="finish-btn" onclick={finish}> Get Started </button>
            {/if}
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
        width: 520px;
        height: 380px;
        background: var(--color-main-bg);
        border: 1px solid var(--color-border-subtle);
        border-radius: 20px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        user-select: none;
        pointer-events: auto;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    }

    .close-btn {
        position: absolute;
        top: 14px;
        left: 14px;
        width: 28px;
        height: 28px;
        border-radius: 50%;
        border: none;
        background: rgba(0, 0, 0, 0.06);
        color: var(--color-text-dim);
        font-size: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 150ms ease;
        z-index: 10;
    }

    .close-btn:hover {
        background: rgba(0, 0, 0, 0.1);
        color: var(--color-text-secondary);
    }

    /* ── Content (integrated) ── */
    .content {
        position: relative;
        z-index: 1;
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 24px;
    }

    .illustration {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }

    .illustration.fullbleed {
        width: 100%;
        height: 100%;
        flex: 1;
        position: relative;
    }

    .illustration.fullbleed::after {
        content: "";
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 60px;
        background: linear-gradient(to top, var(--color-main-bg) 0%, transparent 100%);
        pointer-events: none;
        z-index: 5;
    }

    .text {
        text-align: center;
    }

    .title {
        font-size: 22px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin: 0 0 6px;
        letter-spacing: -0.02em;
    }

    .desc {
        font-size: 14px;
        color: var(--color-text-tertiary);
        margin: 0;
    }

    /* ── Dots (centered at bottom) ── */
    .dots {
        position: absolute;
        bottom: 24px;
        left: 50%;
        transform: translateX(-50%);
        display: flex;
        gap: 8px;
        align-items: center;
        z-index: 10;
    }

    .dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        border: none;
        background: var(--color-button-bg-active);
        cursor: pointer;
        padding: 0;
        transition: all 250ms cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .dot.past {
        background: var(--color-text-dim);
    }

    .dot.active {
        background: var(--color-accent);
        width: 20px;
        border-radius: 3px;
    }

    .dot:hover:not(.active) {
        background: var(--color-text-dim);
    }

    /* ── Navigation Arrows ── */
    .nav-arrow {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        width: 40px;
        height: 40px;
        border-radius: 50%;
        border: none;
        background: rgba(0, 0, 0, 0.04);
        color: var(--color-text-dim);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 150ms ease;
        z-index: 10;
        opacity: 0;
    }

    .card:hover .nav-arrow {
        opacity: 1;
    }

    .nav-arrow:hover {
        background: rgba(0, 0, 0, 0.08);
        color: var(--color-text-secondary);
    }

    .nav-arrow.left {
        left: 16px;
    }

    .nav-arrow.right {
        right: 16px;
    }

    /* ── Finish Button ── */
    .finish-btn {
        position: absolute;
        bottom: 24px;
        right: 24px;
        padding: 10px 24px;
        border-radius: 10px;
        border: none;
        background: var(--color-accent);
        color: #fff;
        font-size: 13px;
        font-weight: 600;
        font-family: inherit;
        cursor: pointer;
        transition: all 150ms ease;
        z-index: 10;
    }

    .finish-btn:hover {
        background: var(--color-accent-hover);
    }

    /* ════════════════════════════════
       ILLUSTRATIONS
    ════════════════════════════════ */

    /* Welcome */
    .welcome-scene {
        position: relative;
        width: 120px;
        height: 120px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .app-icon {
        width: 72px;
        height: 72px;
        border-radius: 16px;
        box-shadow: 0 10px 28px rgba(0, 0, 0, 0.35);
    }

    /* Workspaces — image is the subject, text lives inside the main pane */
    .workspace-layout {
        display: flex;
        gap: 8px;
        width: 85%;
        height: 85%;
        padding: 24px;
        margin-top: 16px;
        transform: translateY(44px);
    }

    .ws-main {
        flex: 1.4;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-medium);
        border-radius: 12px;
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .ws-title {
        font-size: 16px;
        font-weight: 600;
        color: var(--color-text-primary);
        letter-spacing: -0.02em;
    }

    .ws-desc {
        font-size: 11px;
        color: var(--color-text-tertiary);
    }

    .ws-side {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .ws-top,
    .ws-bottom {
        flex: 1;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-medium);
        border-radius: 10px;
    }

    /* Clipboard — clipboard shape as subject, text inside body */
    .clipboard-visual {
        position: relative;
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 32px;
    }

    .clip-clipboard {
        position: relative;
        width: 50%;
        height: 90%;
        transform: translateY(44px);
    }

    .clip-clip {
        position: absolute;
        top: -8px;
        left: 50%;
        transform: translateX(-50%);
        width: 80px;
        height: 20px;
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-medium);
        border-radius: 6px 6px 0 0;
        z-index: 2;
    }

    .clip-clip::before {
        content: "";
        position: absolute;
        top: 6px;
        left: 50%;
        transform: translateX(-50%);
        width: 24px;
        height: 8px;
        border-radius: 4px;
        background: var(--color-border-medium);
    }

    .clip-body {
        width: 100%;
        height: 100%;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-medium);
        border-radius: 12px;
        padding: 32px 20px 20px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 6px;
        text-align: center;
    }

    .clip-title {
        font-size: 16px;
        font-weight: 600;
        color: var(--color-text-primary);
        letter-spacing: -0.02em;
    }

    .clip-desc {
        font-size: 12px;
        color: var(--color-text-tertiary);
    }

    /* Monitor — bars span the full bottom like a waveform, text overlaid */
    .monitor-vis {
        position: relative;
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
    }

    .vis-bars {
        display: flex;
        align-items: flex-end;
        gap: 2px;
        height: 50%;
        width: 100%;
    }

    .vis-bar {
        flex: 1;
        display: flex;
        flex-direction: column;
        border-radius: 2px 2px 0 0;
        overflow: hidden;
        animation: bar-grow 500ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    }

    .vis-bar .bar-top {
        background: var(--color-accent-border);
        flex-shrink: 0;
        height: 30%;
    }

    .vis-bar .bar-bottom {
        background: var(--color-accent-soft);
        flex: 1;
    }

    @keyframes bar-grow {
        from {
            height: 0 !important;
        }
    }

    .monitor-text {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -70%);
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
    }

    .mon-title {
        font-size: 18px;
        font-weight: 600;
        color: var(--color-text-primary);
        letter-spacing: -0.02em;
    }

    .mon-desc {
        font-size: 13px;
        color: var(--color-text-tertiary);
    }

    /* Preferences */
    .prefs-gears {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        overflow: hidden;
    }

    .gear {
        position: absolute;
        color: var(--color-text-dim);
        opacity: 0.3;
    }

    .gear-tr {
        top: -40px;
        right: -40px;
    }

    .gear-bl {
        bottom: -30px;
        left: -30px;
    }

    .prefs-text {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        text-align: center;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .prefs-title {
        font-size: 18px;
        font-weight: 600;
        color: var(--color-text-primary);
        letter-spacing: -0.02em;
    }

    .prefs-desc {
        font-size: 13px;
        color: var(--color-text-tertiary);
    }

    /* Final */
    .final-check {
        position: relative;
        width: 90px;
        height: 90px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .check-icon {
        width: 56px;
        height: 56px;
        border-radius: 50%;
        background: var(--color-success-soft);
        border: 1px solid rgba(87, 211, 140, 0.3);
        color: var(--color-success);
        font-size: 24px;
        font-weight: 700;
        display: flex;
        align-items: center;
        justify-content: center;
    }
</style>