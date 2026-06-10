<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import type { WorkspaceProfile } from "$types/preferences";
    import { applyWorkspace, getWorkspaceProfiles } from "$services/workspaces";

    const COLS = 8;
    const ROWS = 8;
    const MAX_PROFILES = 6;
    const RING_RADIUS = 120;
    const SELECTED_SCALE = 1.15;

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

    function getItemPosition(index: number, total: number) {
        const angle = ((2 * Math.PI) / total) * index - Math.PI / 2;
        const x = Math.cos(angle) * RING_RADIUS;
        const y = Math.sin(angle) * RING_RADIUS;
        return { x, y };
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="root" on:click={close}>
    <div class="ring-container">
        {#each profiles as profile, i}
            {@const pos = getItemPosition(i, profiles.length)}
            {@const isActive = selected === profile.id}

            <div
                class="item"
                class:selected={isActive}
                style="
                    transform: translate({pos.x}px, {pos.y}px) scale({isActive
                    ? SELECTED_SCALE
                    : 1});
                "
                on:mouseenter={() => (selected = profile.id)}
                on:click|stopPropagation={() => apply(profile.id)}
            >
                <div class="thumbnail">
                    {#each profile.apps as app}
                        {@const gx = toGrid(app.x, COLS)}
                        {@const gy = toGrid(app.y, ROWS)}
                        {@const gw = toGrid(app.width, COLS)}
                        {@const gh = toGrid(app.height, ROWS)}

                        <div
                            class="slot"
                            style="
                                left:{pct(gx, COLS)}%;
                                top:{pct(gy, ROWS)}%;
                                width:{pct(gw, COLS)}%;
                                height:{pct(gh, ROWS)}%;
                            "
                        >
                            {#if app.icon}
                                <img
                                    class="icon"
                                    src="data:image/png;base64,{app.icon}"
                                    alt={app.name}
                                />
                            {:else}
                                <span class="initial">
                                    {app.name.charAt(0).toUpperCase()}
                                </span>
                            {/if}
                        </div>
                    {/each}
                </div>

                {#if isActive}
                    <div class="label">{profile.name}</div>
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

    .ring-container {
        position: relative;
        width: 380px;
        height: 380px;
        display: flex;
        align-items: center;
        justify-content: center;
        transform: translateY(-8px);
    }

    .item {
        position: absolute;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
        cursor: pointer;
        transition: transform 200ms cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .thumbnail {
        width: 108px;
        height: 72px;
        background: var(--color-main-bg, #1a1a1a);
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        position: relative;
        overflow: hidden;
        transition: border-color 200ms ease;
    }

    .item.selected .thumbnail {
        border-color: rgba(255, 255, 255, 0.25);
    }

    .slot {
        position: absolute;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 3px;
        display: flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
    }

    .icon {
        width: 18px;
        height: 18px;
        object-fit: contain;
        image-rendering: auto;
    }

    .initial {
        font-size: 7px;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.85);
    }

    .label {
        font-size: 11px;
        font-weight: 500;
        color: var(--color-text-primary, #fff);
        text-align: center;
        white-space: nowrap;
        position: absolute;
        bottom: -18px;
        left: 50%;
        transform: translateX(-50%);
    }
</style>
