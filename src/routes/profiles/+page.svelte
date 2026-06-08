<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import type { WorkspaceProfile } from "$types/preferences";
    import { applyWorkspace, getWorkspaceProfiles } from "$services/workspaces";

    const COLS = 12;
    const ROWS = 8;
    const MAX_PROFILES = 6;

    let profiles: WorkspaceProfile[] = [];
    let selected: string | null = null;
    let applying = false;

    function toGrid(val: number, total: number) {
        return Math.round(val * total);
    }

    function pct(val: number, total: number) {
        return (val / total) * 100;
    }

    async function loadProfiles() {
        profiles = await getWorkspaceProfiles();
        profiles = profiles.slice(0, MAX_PROFILES);

        if (!selected || !profiles.some((p) => p.id === selected)) {
            selected = profiles[0]?.id ?? null;
        }
    }

    onMount(async () => {
        await loadProfiles();

        window.addEventListener("keydown", onKey);
    });

    onDestroy(() => {
        window.removeEventListener("keydown", onKey);
    });

    function onKey(e: KeyboardEvent) {
        if (e.key === "Escape") {
            close();
            return;
        }
        if (e.key === "Enter" && selected) {
            apply(selected);
            return;
        }
        if (e.key === "ArrowRight" || e.key === "ArrowDown") {
            const idx = profiles.findIndex((p) => p.id === selected);

            selected = profiles[(idx + 1) % profiles.length].id;

            return;
        }
        if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
            const idx = profiles.findIndex((p) => p.id === selected);

            selected =
                profiles[(idx - 1 + profiles.length) % profiles.length].id;

            return;
        }
    }

    async function apply(profileId: string) {
        if (applying) return;
        applying = true;
        await applyWorkspace(profileId);
        applying = false;
        await close();
    }

    async function close() {
        await getCurrentWindow().hide();
    }

    function polarToCartesian(
        cx: number,
        cy: number,
        radius: number,
        angle: number,
    ) {
        const rad = ((angle - 90) * Math.PI) / 180;

        return {
            x: cx + radius * Math.cos(rad),
            y: cy + radius * Math.sin(rad),
        };
    }

    function describeArc(
        cx: number,
        cy: number,
        radius: number,
        startAngle: number,
        endAngle: number,
    ) {
        const start = polarToCartesian(cx, cy, radius, startAngle);

        const end = polarToCartesian(cx, cy, radius, endAngle);

        const largeArc = endAngle - startAngle <= 180 ? 0 : 1;

        return `
            M ${start.x} ${start.y}
            A ${radius} ${radius}
              0 ${largeArc} 1
              ${end.x} ${end.y}
        `;
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="root">
    <div class="wheel-container">
        <div class="selector-backdrop"></div>
        <svg class="wheel" viewBox="0 0 380 380">
            {#each profiles as profile, i}
                {@const startAngle = (360 / profiles.length) * i}

                {@const endAngle = (360 / profiles.length) * (i + 1)}

                {@const path = describeArc(190, 190, 176, startAngle, endAngle)}

                <path
                    d={path}
                    class="arc"
                    class:selected={selected === profile.id}
                    on:mouseenter={() => (selected = profile.id)}
                    on:click={() => apply(profile.id)}
                />
            {/each}
        </svg>

        {#each profiles as profile, i}
            {@const startAngle = (360 / profiles.length) * i}
            {@const endAngle = (360 / profiles.length) * (i + 1)}
            {@const midAngle = (startAngle + endAngle) / 2 - 90}

            {@const rad = (midAngle * Math.PI) / 180}

            {@const tx = 190 + Math.cos(rad) * 115}

            {@const ty = 190 + Math.sin(rad) * 115}

            <div
                class="slice-preview"
                class:selected={selected === profile.id}
                on:mouseenter={() => (selected = profile.id)}
                on:click={() => apply(profile.id)}
                style="
                           left:{tx}px;
                           top:{ty}px;
                       "
            >
                <div class="preview">
                    {#each profile.apps as app}
                        {@const gx = toGrid(app.x, COLS)}
                        {@const gy = toGrid(app.y, ROWS)}
                        {@const gw = toGrid(app.width, COLS)}
                        {@const gh = toGrid(app.height, ROWS)}

                        <div
                            class="preview-slot"
                            style="
                                       left:{pct(gx, COLS)}%;
                                       top:{pct(gy, ROWS)}%;
                                       width:{pct(gw, COLS)}%;
                                       height:{pct(gh, ROWS)}%;
                                   "
                        >
                            {#if app.icon}
                                <img
                                    class="preview-icon"
                                    src="data:image/png;base64,{app.icon}"
                                    alt={app.name}
                                />
                            {:else}
                                <span class="preview-initial">
                                    {app.name.charAt(0).toUpperCase()}
                                </span>
                            {/if}
                        </div>
                    {/each}
                </div>

                {#if selected === profile.id}
                    <div class="slice-name">
                        {profile.name}
                    </div>
                {/if}
            </div>
        {/each}
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

    .wheel-container {
        position: relative;

        width: 380px;
        height: 380px;
    }

    .selector-backdrop {
        position: absolute;
        inset: 20px;

        border-radius: 50%;

        backdrop-filter: blur(24px);

        background: color-mix(in srgb, var(--color-main-bg) 35%, transparent);

        border: 1px solid rgba(255, 255, 255, 0.08);

        pointer-events: none;
    }

    .wheel {
        position: absolute;
        inset: 0;

        width: 100%;
        height: 100%;

        z-index: 1;
    }

    .arc {
        fill: none;

        stroke: rgba(255, 255, 255, 0.12);

        stroke-width: 1.5;

        stroke-linecap: round;

        opacity: 0.6;

        transition: all 200ms ease;
    }

    .arc.selected {
        stroke: var(--color-accent);

        stroke-width: 3;

        opacity: 1;
    }

    .slice-preview {
        position: absolute;

        left: 0;
        top: 0;

        display: flex;
        flex-direction: column;
        align-items: center;

        transform: translate(-50%, -50%) scale(0.85);

        opacity: 0.5;

        transition: all 250ms cubic-bezier(0.2, 0.8, 0.2, 1);

        z-index: 2;
    }

    .slice-preview.selected {
        transform: translate(-50%, -50%) scale(1.12);

        opacity: 1;
    }
    .slice-preview.selected .preview-icon {
        width: 24px;
        height: 24px;
    }

    .preview {
        position: relative;

        width: 90px;
        height: 54px;

        background: var(--color-main-bg);

        border-radius: 6px;

        border: 1px solid transparent;

        overflow: hidden;

        transition: all 250ms cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .preview-slot {
        position: absolute;

        background: rgba(255, 255, 255, 0.08);

        border: 1px solid rgba(255, 255, 255, 0.18);

        border-radius: 4px;

        display: flex;
        align-items: center;
        justify-content: center;

        box-sizing: border-box;
    }

    .preview-icon {
        width: 18px;
        height: 18px;

        object-fit: contain;

        image-rendering: auto;

        transition:
            width 180ms cubic-bezier(0.2, 0.8, 0.2, 1),
            height 180ms cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .preview-initial {
        font-size: 8px;
        font-weight: 700;

        color: rgba(255, 255, 255, 0.9);
    }

    .slice-name {
        font-size: 11px;
        font-weight: 500;

        color: var(--color-text-primary);

        text-align: center;
    }
</style>
