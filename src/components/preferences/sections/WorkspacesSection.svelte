<script lang="ts">
    import { onMount } from "svelte";
    import PreferencesSection from "../PreferencesSection.svelte";
    import AppPickerModal from "$components/clipboard/AppPickerModal.svelte";
    import { canUse, license } from "$stores/license";
    import type { WorkspaceApp, WorkspaceProfile } from "$types/preferences";
    import type { IgnoredApp } from "$types/preferences";
    import {
        getWorkspaceProfiles,
        saveWorkspaceProfile,
        deleteWorkspaceProfile,
        applyWorkspace,
    } from "$services/workspaces";

    $: isPro = canUse("workspace_profiles", $license);

    let profiles: WorkspaceProfile[] = [];
    let selected: WorkspaceProfile | null = null;
    let loading = true;
    let saving = false;
    let applyingId: string | null = null;
    let showAppPicker = false;

    // drag state
    let dragging: WorkspaceApp | null = null;
    let dragMode: "move" | "resize" = "move";
    let dragStart = { mx: 0, my: 0, ox: 0, oy: 0, ow: 0, oh: 0 };
    let canvasEl: HTMLDivElement;

    onMount(async () => {
        try {
            profiles = await getWorkspaceProfiles();
        } finally {
            loading = false;
        }
    });

    function newProfile() {
        const id = crypto.randomUUID();
        const profile: WorkspaceProfile = { id, name: "New Profile", apps: [] };
        profiles = [...profiles, profile];
        selected = profile;
    }

    async function saveSelected() {
        if (!selected) return;
        saving = true;
        try {
            await saveWorkspaceProfile(selected);
            profiles = profiles.map((p) =>
                p.id === selected!.id ? selected! : p,
            );
        } catch (e) {
            console.error(e);
        } finally {
            saving = false;
        }
    }

    async function deleteSelected() {
        if (!selected) return;
        await deleteWorkspaceProfile(selected.id);
        profiles = profiles.filter((p) => p.id !== selected!.id);
        selected = profiles[0] ?? null;
    }

    async function handleApply(profileId: string) {
        applyingId = profileId;
        try {
            await applyWorkspace(profileId);
        } catch (e) {
            console.error(e);
        } finally {
            applyingId = null;
        }
    }

    function onAppPicked(app: IgnoredApp) {
        if (!selected) return;
        const newApp: WorkspaceApp = {
            name: app.name,
            path: app.path,
            exe_path: app.exe_path,
            icon: app.icon,
            x: 0,
            y: 0,
            width: 0.4,
            height: 0.5,
        };
        selected = { ...selected, apps: [...selected.apps, newApp] };
        showAppPicker = false;
    }

    function removeApp(index: number) {
        if (!selected) return;
        const apps = [...selected.apps];
        apps.splice(index, 1);
        selected = { ...selected, apps };
    }

    function startDrag(
        e: MouseEvent,
        app: WorkspaceApp,
        mode: "move" | "resize",
    ) {
        if (!isPro || !canvasEl) return;
        e.preventDefault();
        dragging = app;
        dragMode = mode;
        dragStart = {
            mx: e.clientX,
            my: e.clientY,
            ox: app.x,
            oy: app.y,
            ow: app.width,
            oh: app.height,
        };
        window.addEventListener("mousemove", onDrag);
        window.addEventListener("mouseup", stopDrag);
    }

    function onDrag(e: MouseEvent) {
        if (!dragging || !selected || !canvasEl) return;
        const rect = canvasEl.getBoundingClientRect();
        const dx = (e.clientX - dragStart.mx) / rect.width;
        const dy = (e.clientY - dragStart.my) / rect.height;

        const app = dragging;
        if (dragMode === "move") {
            app.x = Math.max(0, Math.min(1 - app.width, dragStart.ox + dx));
            app.y = Math.max(0, Math.min(1 - app.height, dragStart.oy + dy));
            clampApp(app);
        } else {
            app.width = Math.max(0.1, Math.min(1 - app.x, dragStart.ow + dx));
            app.height = Math.max(0.1, Math.min(1 - app.y, dragStart.oh + dy));
            clampApp(app);
        }
        selected = { ...selected, apps: [...selected.apps] };
    }

    function clampApp(app: WorkspaceApp) {
        app.width = Math.max(0.1, Math.min(1, app.width));
        app.height = Math.max(0.1, Math.min(1, app.height));

        app.x = Math.max(0, Math.min(1 - app.width, app.x));
        app.y = Math.max(0, Math.min(1 - app.height, app.y));
    }

    function stopDrag() {
        dragging = null;
        window.removeEventListener("mousemove", onDrag);
        window.removeEventListener("mouseup", stopDrag);
    }

    const PALETTE = [
        "#4C8EF7",
        "#F7824C",
        "#4CF7A0",
        "#F7E04C",
        "#C44CF7",
        "#4CF0F7",
    ];
    function colorFor(i: number) {
        return PALETTE[i % PALETTE.length];
    }
</script>

{#if !isPro}
    <PreferencesSection
        title="Workspace Profiles"
        description="Save and restore complete app layouts"
    >
        <div class="pro-gate">
            <svg
                width="32"
                height="32"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
            >
                <rect x="3" y="11" width="18" height="11" rx="2" />
                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            <p class="pro-title">Pro feature</p>
            <p class="pro-desc">
                Workspace Profiles let you save app arrangements and restore
                them instantly. Upgrade to Pro to unlock.
            </p>
            <button class="upgrade-btn">Upgrade to Pro</button>
        </div>
    </PreferencesSection>
{:else}
    <PreferencesSection
        title="Workspace Profiles"
        description="Save and restore complete app layouts with one click"
    >
        <div class="workspaces-root">
            <div class="profile-list">
                {#if loading}
                    <div class="list-empty">Loading…</div>
                {:else if profiles.length === 0}
                    <div class="list-empty">No profiles yet</div>
                {:else}
                    {#each profiles as p}
                        <button
                            class="profile-row"
                            class:active={selected?.id === p.id}
                            on:click={() => (selected = p)}
                        >
                            <span class="profile-name">{p.name}</span>
                            <span class="app-count"
                                >{p.apps.length} app{p.apps.length !== 1
                                    ? "s"
                                    : ""}</span
                            >
                        </button>
                    {/each}
                {/if}

                <button class="new-profile-btn" on:click={newProfile}>
                    <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                        stroke-linecap="round"
                    >
                        <line x1="12" y1="5" x2="12" y2="19" /><line
                            x1="5"
                            y1="12"
                            x2="19"
                            y2="12"
                        />
                    </svg>
                    New profile
                </button>
            </div>

            {#if selected}
                <div class="profile-editor">
                    <div class="editor-header">
                        <input
                            class="name-input"
                            bind:value={selected.name}
                            placeholder="Profile name"
                        />
                        <div class="header-actions">
                            <button
                                class="apply-btn"
                                disabled={applyingId === selected.id}
                                on:click={() => handleApply(selected!.id)}
                            >
                                {applyingId === selected.id
                                    ? "Launching…"
                                    : "▶ Apply"}
                            </button>
                            <button
                                class="save-btn"
                                disabled={saving}
                                on:click={saveSelected}
                            >
                                {saving ? "Saving…" : "Save"}
                            </button>
                            <button
                                class="delete-btn"
                                on:click={deleteSelected}
                                aria-label="Delete profile"
                            >
                                <svg
                                    width="13"
                                    height="13"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                >
                                    <polyline points="3 6 5 6 21 6" /><path
                                        d="M19 6l-1 14H6L5 6"
                                    /><path d="M10 11v6" /><path
                                        d="M14 11v6"
                                    /><path d="M9 6V4h6v2" />
                                </svg>
                            </button>
                        </div>
                    </div>

                    <div class="canvas-wrap">
                        <div class="canvas-label">
                            Layout <span class="canvas-hint"
                                >drag to move · corner to resize</span
                            >
                        </div>
                        <div class="canvas" bind:this={canvasEl}>
                            <!-- screen grid lines -->
                            <div class="grid-line v" style="left:33.3%"></div>
                            <div class="grid-line v" style="left:66.6%"></div>
                            <div class="grid-line h" style="top:50%"></div>

                            {#each selected.apps as app, i}
                                <!-- svelte-ignore a11y-no-static-element-interactions -->
                                <div
                                    class="app-slot"
                                    style="
                                left:{app.x * 100}%;
                                top:{app.y * 100}%;
                                width:{app.width * 100}%;
                                height:{app.height * 100}%;
                                --slot-color:{colorFor(i)};
                                "
                                    on:mousedown={(e) =>
                                        startDrag(e, app, "move")}
                                >
                                    {#if app.icon}
                                        <img
                                            class="slot-icon"
                                            src="data:image/png;base64,{app.icon}"
                                            alt=""
                                        />
                                    {/if}
                                    <span class="slot-name">{app.name}</span>
                                    <button
                                        class="slot-remove"
                                        on:click|stopPropagation={() =>
                                            removeApp(i)}
                                        aria-label="Remove {app.name}">×</button
                                    >
                                    <!-- resize handle -->
                                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                                    <div
                                        class="resize-handle"
                                        on:mousedown|stopPropagation={(e) =>
                                            startDrag(e, app, "resize")}
                                    ></div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </div>

                <!-- app list below canvas -->
                <div class="app-list-header">
                    <span class="app-list-title">Apps in this profile</span>
                    <button
                        class="add-app-btn"
                        on:click={() => (showAppPicker = true)}
                    >
                        <svg
                            width="11"
                            height="11"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                            stroke-linecap="round"
                        >
                            <line x1="12" y1="5" x2="12" y2="19" /><line
                                x1="5"
                                y1="12"
                                x2="19"
                                y2="12"
                            />
                        </svg>
                        Add app
                    </button>
                </div>

                {#if selected.apps.length === 0}
                    <div class="apps-empty">
                        Add apps to define this workspace
                    </div>
                {:else}
                    <div class="app-rows">
                        {#each selected.apps as app, i}
                            <div class="app-row">
                                <span
                                    class="app-dot"
                                    style="background:{colorFor(i)}"
                                ></span>
                                {#if app.icon}
                                    <img
                                        class="app-icon"
                                        src="data:image/png;base64,{app.icon}"
                                        alt=""
                                    />
                                {:else}
                                    <div class="app-icon placeholder"></div>
                                {/if}
                                <span class="app-name">{app.name}</span>
                                <span class="app-coords">
                                    {Math.round(app.x * 100)}%,{Math.round(
                                        app.y * 100,
                                    )}% · {Math.round(
                                        app.width * 100,
                                    )}×{Math.round(app.height * 100)}%
                                </span>
                                <button
                                    class="row-remove"
                                    on:click={() => removeApp(i)}
                                    aria-label="Remove">×</button
                                >
                            </div>
                        {/each}
                    </div>
                {/if}
            {:else}
                <div class="no-selection">
                    Select a profile or create one to get started
                </div>
            {/if}
        </div>
    </PreferencesSection>
{/if}

{#if showAppPicker}
    <AppPickerModal
        onpick={onAppPicked}
        onclose={() => (showAppPicker = false)}
    />
{/if}

<style>
    .workspaces-root {
        display: grid;
        grid-template-columns: 180px 1fr;
        gap: 0;
        min-height: 420px;
        border: 1px solid var(--color-border-subtle);
        border-radius: var(--radius-md);
        overflow: hidden;
    }

    .profile-list {
        border-right: 1px solid var(--color-border-subtle);
        display: flex;
        flex-direction: column;
        background: var(--color-surface);
    }

    .list-empty {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        color: var(--color-text-dim);
        padding: 16px;
        text-align: center;
    }

    .profile-row {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 10px 12px;
        border: none;
        background: transparent;
        text-align: left;
        cursor: pointer;
        border-bottom: 1px solid var(--color-border-subtle);
        transition: background var(--transition-fast);
    }
    .profile-row:hover {
        background: var(--color-button-bg-hover);
    }
    .profile-row.active {
        background: var(--color-button-bg);
    }

    .profile-name {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-primary);
    }
    .app-count {
        font-size: 11px;
        color: var(--color-text-dim);
    }

    .new-profile-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        margin: 8px;
        padding: 7px 10px;
        border-radius: var(--radius-sm);
        border: 1px dashed var(--color-border-medium);
        background: transparent;
        color: var(--color-text-dim);
        font-size: 12px;
        font-family: inherit;
        cursor: pointer;
        transition: all var(--transition-fast);
    }
    .new-profile-btn:hover {
        color: var(--color-text-primary);
        border-color: var(--color-border-strong);
        background: var(--color-button-bg);
    }

    .profile-editor {
        display: flex;
        flex-direction: column;
        gap: 0;
        overflow-y: auto;
        scrollbar-width: none;
    }

    .editor-header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 12px 14px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .name-input {
        flex: 1;
        padding: 6px 10px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-primary);
        font-size: 13px;
        font-family: inherit;
        font-weight: 500;
    }
    .name-input:focus {
        outline: none;
        border-color: var(--color-accent-border);
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .apply-btn,
    .save-btn {
        padding: 6px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 12px;
        font-family: inherit;
        font-weight: 500;
        cursor: pointer;
        transition: all var(--transition-fast);
    }
    .apply-btn {
        border-color: var(--color-accent-border);
        color: var(--color-accent);
    }
    .apply-btn:hover:not(:disabled) {
        background: var(--color-accent);
        color: #fff;
    }
    .save-btn:hover:not(:disabled) {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }
    .apply-btn:disabled,
    .save-btn:disabled {
        opacity: 0.5;
        cursor: default;
    }

    .delete-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        border: none;
        background: transparent;
        color: var(--color-text-dim);
        border-radius: var(--radius-sm);
        cursor: pointer;
        transition: all var(--transition-fast);
    }
    .delete-btn:hover {
        background: var(--color-button-bg-hover);
        color: #ef4444;
    }

    .canvas-wrap {
        padding: 14px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .canvas-label {
        font-size: 11px;
        font-weight: 600;
        color: var(--color-text-dim);
        text-transform: uppercase;
        letter-spacing: 0.06em;
        margin-bottom: 8px;
    }
    .canvas-hint {
        font-weight: 400;
        text-transform: none;
        letter-spacing: 0;
        margin-left: 6px;
        opacity: 0.7;
    }

    .canvas {
        position: relative;
        width: 100%;
        aspect-ratio: 16/9;
        background: var(--color-surface);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-sm);
        overflow: hidden;
        user-select: none;
    }

    .grid-line {
        position: absolute;
        background: var(--color-border-subtle);
        pointer-events: none;
    }
    .grid-line.v {
        top: 0;
        bottom: 0;
        width: 1px;
    }
    .grid-line.h {
        left: 0;
        right: 0;
        height: 1px;
    }

    .app-slot {
        position: absolute;
        background: color-mix(in srgb, var(--slot-color) 18%, transparent);
        border: 2px solid var(--slot-color);
        border-radius: 4px;
        display: flex;
        align-items: flex-start;
        gap: 4px;
        padding: 4px 6px;
        cursor: grab;
        overflow: hidden;
        min-width: 0;
        box-sizing: border-box;
    }
    .app-slot:active {
        cursor: grabbing;
    }

    .slot-icon {
        width: 14px;
        height: 14px;
        flex-shrink: 0;
        border-radius: 2px;
        object-fit: contain;
        margin-top: 1px;
    }

    .slot-name {
        font-size: 10px;
        font-weight: 600;
        color: var(--slot-color);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
        min-width: 0;
    }

    .slot-remove {
        flex-shrink: 0;
        background: none;
        border: none;
        color: var(--slot-color);
        font-size: 13px;
        line-height: 1;
        padding: 0 2px;
        cursor: pointer;
        opacity: 0.7;
    }
    .slot-remove:hover {
        opacity: 1;
    }

    .resize-handle {
        position: absolute;
        bottom: 0;
        right: 0;
        width: 12px;
        height: 12px;
        cursor: se-resize;
        background: linear-gradient(
            135deg,
            transparent 50%,
            var(--slot-color) 50%
        );
        opacity: 0.8;
    }

    .app-list-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 14px 6px;
    }
    .app-list-title {
        font-size: 11px;
        font-weight: 600;
        color: var(--color-text-dim);
        text-transform: uppercase;
        letter-spacing: 0.06em;
    }
    .add-app-btn {
        display: inline-flex;
        align-items: center;
        gap: 5px;
        padding: 5px 10px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 12px;
        font-family: inherit;
        cursor: pointer;
        transition: all var(--transition-fast);
    }
    .add-app-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .apps-empty {
        padding: 12px 14px;
        font-size: 12px;
        color: var(--color-text-dim);
    }

    .app-rows {
        display: flex;
        flex-direction: column;
        padding: 0 14px 14px;
        gap: 4px;
    }

    .app-row {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 8px;
        border-radius: var(--radius-sm);
        background: var(--color-button-bg);
    }
    .app-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        flex-shrink: 0;
    }
    .app-icon {
        width: 18px;
        height: 18px;
        border-radius: 3px;
        object-fit: contain;
        flex-shrink: 0;
    }
    .app-icon.placeholder {
        background: var(--color-border-subtle);
    }
    .app-name {
        font-size: 12px;
        font-weight: 500;
        color: var(--color-text-secondary);
        flex: 1;
        min-width: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .app-coords {
        font-size: 11px;
        color: var(--color-text-dim);
        white-space: nowrap;
    }
    .row-remove {
        background: none;
        border: none;
        color: var(--color-text-dim);
        font-size: 14px;
        cursor: pointer;
        padding: 0 2px;
        line-height: 1;
    }
    .row-remove:hover {
        color: #ef4444;
    }

    .no-selection {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        font-size: 13px;
        color: var(--color-text-dim);
        padding: 32px;
        text-align: center;
    }

    .pro-gate {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        padding: 40px 24px;
        text-align: center;
        color: var(--color-text-dim);
    }
    .pro-title {
        font-size: 15px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin: 0;
    }
    .pro-desc {
        font-size: 13px;
        color: var(--color-text-secondary);
        max-width: 320px;
        margin: 0;
        line-height: 1.5;
    }
    .upgrade-btn {
        margin-top: 6px;
        padding: 8px 20px;
        border-radius: var(--radius-sm);
        border: none;
        background: var(--color-accent);
        color: #fff;
        font-size: 13px;
        font-weight: 600;
        font-family: inherit;
        cursor: pointer;
        transition: opacity var(--transition-fast);
    }
    .upgrade-btn:hover {
        opacity: 0.85;
    }
</style>
