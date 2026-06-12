<script lang="ts">
    import { onMount } from "svelte";
    import PreferencesSection from "../PreferencesSection.svelte";
    import AppPickerModal from "$components/clipboard/AppPickerModal.svelte";
    import BrowserUrlModal from "$components/workspaces/BrowserUrlModal.svelte";
    import { canUse, license } from "$stores/license";
    import type { WorkspaceApp, WorkspaceProfile } from "$types/preferences";
    import type { IgnoredApp } from "$types/preferences";
    import {
        getWorkspaceProfiles,
        saveWorkspaceProfile,
        deleteWorkspaceProfile,
    } from "$services/workspaces";
    import { openUrl } from "@tauri-apps/plugin-opener";

    $: isPro = canUse("workspace_profiles", $license);

    const COLS = 12;
    const ROWS = 8;

    const BROWSER_KEYWORDS = [
        "chrome", "firefox", "edge", "arc", "zen", "brave", "opera",
        "vivaldi", "waterfox", "librewolf", "browser", "safari",
    ];

    function isBrowserApp(app: WorkspaceApp): boolean {
        const name = app.name.toLowerCase();
        const path = (app.exe_path ?? app.path).toLowerCase();
        return BROWSER_KEYWORDS.some(
            (k) => name.includes(k) || path.includes(k),
        );
    }

    let profiles: WorkspaceProfile[] = [];
    let selected: WorkspaceProfile | null = null;
    let loading = true;
    let saving = false;
    let showAppPicker = false;
    let noSpaceError = false;

    // URL modal state
    let showUrlModal = false;
    let urlModalAppName = "";
    let urlModalAppIndex: number | null = null;
    let urlModalUrls: string[] = [];

    // drag state
    let dragging: WorkspaceApp | null = null;
    let dragMode: "move" | "resize" = "move";
    let resizeCorner: "nw" | "ne" | "sw" | "se" = "se";
    let dragStartCell = { col: 0, row: 0 };
    let dragStartApp = { x: 0, y: 0, width: 0, height: 0 };
    let canvasEl: HTMLDivElement;

    async function upgrade() {
        openUrl("https://appkova.com/#pricing");
    }

    onMount(async () => {
        try {
            profiles = await getWorkspaceProfiles();
            // Convert percentages to grid cells for UI
            profiles = profiles.map((p) => ({
                ...p,
                gap: p.gap ?? 0,
                apps: p.apps.map((app) => ({
                    ...app,
                    x: percentToGrid(app.x, COLS),
                    y: percentToGrid(app.y, ROWS),
                    width: percentToGrid(app.width, COLS),
                    height: percentToGrid(app.height, ROWS),
                })),
            }));
            if (profiles.length > 0) selected = profiles[0];
        } finally {
            loading = false;
        }
    });

    function getCellFromMouse(e: MouseEvent): { col: number; row: number } {
        const rect = canvasEl.getBoundingClientRect();
        const col = Math.floor(((e.clientX - rect.left) / rect.width) * COLS);
        const row = Math.floor(((e.clientY - rect.top) / rect.height) * ROWS);
        return {
            col: Math.max(0, Math.min(COLS - 1, col)),
            row: Math.max(0, Math.min(ROWS - 1, row)),
        };
    }

    function findFreeRegion(
        apps: WorkspaceApp[],
        w: number,
        h: number,
    ): { x: number; y: number } | null {
        for (let row = 0; row <= ROWS - h; row++) {
            for (let col = 0; col <= COLS - w; col++) {
                if (!overlapsAny(apps, col, row, w, h, null)) {
                    return { x: col, y: row };
                }
            }
        }
        return null;
    }

    function overlapsAny(
        apps: WorkspaceApp[],
        x: number,
        y: number,
        w: number,
        h: number,
        exclude: WorkspaceApp | null,
    ): boolean {
        return apps.some((a) => {
            if (a === exclude) return false;
            return !(
                x >= a.x + a.width ||
                x + w <= a.x ||
                y >= a.y + a.height ||
                y + h <= a.y
            );
        });
    }

    function newProfile() {
        const id = crypto.randomUUID();
        const profile: WorkspaceProfile = {
            id,
            name: "New Profile",
            gap: 0,
            apps: [],
        };
        profiles = [...profiles, profile];
        selected = profile;
    }

    async function saveSelected() {
        if (!selected) return;
        saving = true;
        try {
            // Convert grid cells to percentages for backend
            const profileToSave = {
                ...selected,
                apps: selected.apps.map((app) => ({
                    ...app,
                    x: gridToPercent(app.x, COLS),
                    y: gridToPercent(app.y, ROWS),
                    width: gridToPercent(app.width, COLS),
                    height: gridToPercent(app.height, ROWS),
                })),
            };
            await saveWorkspaceProfile(profileToSave);
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

    function onAppPicked(app: IgnoredApp) {
        if (!selected) return;

        const sizes = [
            { w: 6, h: 8 },
            { w: 6, h: 4 },
            { w: 4, h: 8 },
            { w: 4, h: 4 },
            { w: 4, h: 2 },
            { w: 2, h: 4 },
            { w: 3, h: 3 },
            { w: 2, h: 2 },
            { w: 1, h: 1 },
        ];

        let pos: { x: number; y: number } | null = null;
        let chosenW = 4;
        let chosenH = 4;

        for (const { w, h } of sizes) {
            pos = findFreeRegion(selected.apps, w, h);
            if (pos) {
                chosenW = w;
                chosenH = h;
                break;
            }
        }

        if (!pos) {
            noSpaceError = true;
            setTimeout(() => (noSpaceError = false), 3000);
            showAppPicker = false;
            return;
        }

        noSpaceError = false;
        const newApp: WorkspaceApp = {
            name: app.name,
            path: app.path,
            exe_path: app.exe_path,
            icon: app.icon,
            x: pos.x,
            y: pos.y,
            width: chosenW,
            height: chosenH,
        };
        selected = { ...selected, apps: [...selected.apps, newApp] };
        showAppPicker = false;
        saveSelected();

        if (isBrowserApp(newApp)) {
            const idx = selected.apps.length - 1;
            openUrlModal(idx);
        }
    }

    function removeApp(index: number) {
        if (!selected) return;
        const apps = [...selected.apps];
        apps.splice(index, 1);
        selected = { ...selected, apps };
    }

    function openUrlModal(index: number) {
        if (!selected) return;
        const app = selected.apps[index];
        if (!app) return;
        urlModalAppName = app.name;
        urlModalAppIndex = index;
        urlModalUrls = [...(app.urls ?? [])];
        showUrlModal = true;
    }

    function onUrlSave(urls: string[]) {
        if (!selected || urlModalAppIndex === null) return;
        const apps = [...selected.apps];
        apps[urlModalAppIndex] = { ...apps[urlModalAppIndex], urls };
        selected = { ...selected, apps };
        saveSelected();
    }

    function startDrag(
        e: MouseEvent,
        app: WorkspaceApp,
        mode: "move" | "resize",
        corner: "nw" | "ne" | "sw" | "se" = "se",
    ) {
        if (!isPro || !canvasEl) return;
        e.preventDefault();
        dragging = app;
        dragMode = mode;
        resizeCorner = corner;
        dragStartCell = getCellFromMouse(e);
        dragStartApp = {
            x: app.x,
            y: app.y,
            width: app.width,
            height: app.height,
        };
        window.addEventListener("mousemove", onDrag);
        window.addEventListener("mouseup", stopDrag);
    }

    function onDrag(e: MouseEvent) {
        if (!dragging || !selected || !canvasEl) return;
        const cell = getCellFromMouse(e);
        const dcol = cell.col - dragStartCell.col;
        const drow = cell.row - dragStartCell.row;

        let newX = dragStartApp.x;
        let newY = dragStartApp.y;
        let newW = dragStartApp.width;
        let newH = dragStartApp.height;

        if (dragMode === "move") {
            newX = Math.max(
                0,
                Math.min(COLS - dragStartApp.width, dragStartApp.x + dcol),
            );
            newY = Math.max(
                0,
                Math.min(ROWS - dragStartApp.height, dragStartApp.y + drow),
            );
        } else {
            // Handle different resize corners
            switch (resizeCorner) {
                case "se": // bottom-right
                    newW = Math.max(
                        1,
                        Math.min(
                            COLS - dragStartApp.x,
                            dragStartApp.width + dcol,
                        ),
                    );
                    newH = Math.max(
                        1,
                        Math.min(
                            ROWS - dragStartApp.y,
                            dragStartApp.height + drow,
                        ),
                    );
                    break;
                case "sw": // bottom-left
                    newX = Math.max(
                        0,
                        Math.min(
                            dragStartApp.x + dragStartApp.width - 1,
                            dragStartApp.x + dcol,
                        ),
                    );
                    newW = dragStartApp.width + (dragStartApp.x - newX);
                    newH = Math.max(
                        1,
                        Math.min(
                            ROWS - dragStartApp.y,
                            dragStartApp.height + drow,
                        ),
                    );
                    break;
                case "ne": // top-right
                    newY = Math.max(
                        0,
                        Math.min(
                            dragStartApp.y + dragStartApp.height - 1,
                            dragStartApp.y + drow,
                        ),
                    );
                    newH = dragStartApp.height + (dragStartApp.y - newY);
                    newW = Math.max(
                        1,
                        Math.min(
                            COLS - dragStartApp.x,
                            dragStartApp.width + dcol,
                        ),
                    );
                    break;
                case "nw": // top-left
                    newX = Math.max(
                        0,
                        Math.min(
                            dragStartApp.x + dragStartApp.width - 1,
                            dragStartApp.x + dcol,
                        ),
                    );
                    newY = Math.max(
                        0,
                        Math.min(
                            dragStartApp.y + dragStartApp.height - 1,
                            dragStartApp.y + drow,
                        ),
                    );
                    newW = dragStartApp.width + (dragStartApp.x - newX);
                    newH = dragStartApp.height + (dragStartApp.y - newY);
                    break;
            }
        }

        // Only apply if no overlap with other apps
        if (!overlapsAny(selected.apps, newX, newY, newW, newH, dragging)) {
            dragging.x = newX;
            dragging.y = newY;
            dragging.width = newW;
            dragging.height = newH;
            selected = { ...selected, apps: [...selected.apps] };
        }
    }

    function stopDrag() {
        dragging = null;
        window.removeEventListener("mousemove", onDrag);
        window.removeEventListener("mouseup", stopDrag);
    }

    // Convert grid coords to percentage for CSS
    function toPercent(val: number, total: number) {
        return (val / total) * 100;
    }

    // Convert grid cells to percentages for backend
    function gridToPercent(val: number, total: number) {
        return val / total;
    }

    // Convert percentages from backend to grid cells
    function percentToGrid(val: number, total: number) {
        return Math.round(val * total);
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
            <button class="upgrade-btn" on:click={upgrade}
                >Upgrade to Pro</button
            >
        </div>
    </PreferencesSection>
{:else}
    <PreferencesSection
        title="Workspace Profiles"
        description="Save and restore complete app layouts with one click"
    >
        <div class="workspaces-root">
            <!-- Sidebar -->
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
                    New profile
                </button>
            </div>

            <!-- Editor -->
            {#if selected}
                <div class="profile-editor">
                    <div class="editor-header">
                        <input
                            class="name-input"
                            bind:value={selected.name}
                            placeholder="Profile name"
                        />
                        <div class="gap-control">
                            <span class="gap-label">Gap</span>
                            <input
                                type="range"
                                class="gap-slider"
                                min="0"
                                max="40"
                                step="4"
                                bind:value={selected.gap}
                            />
                            <span class="gap-value">{selected.gap ?? 0}px</span>
                        </div>
                        <div class="header-actions">
                            {#if noSpaceError}
                                <span class="no-space-msg"
                                    >No space left in layout</span
                                >
                            {/if}
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

                    <!-- Canvas -->
                    <div class="canvas-wrap">
                        <!-- svelte-ignore a11y-no-static-element-interactions -->
                        <div class="canvas" bind:this={canvasEl}>
                            <!-- Grid cells -->
                            {#each Array(ROWS) as _, row}
                                {#each Array(COLS) as _, col}
                                    <div
                                        class="grid-cell"
                                        style="
                                            left:{toPercent(col, COLS)}%;
                                            top:{toPercent(row, ROWS)}%;
                                            width:{toPercent(1, COLS)}%;
                                            height:{toPercent(1, ROWS)}%;
                                        "
                                    ></div>
                                {/each}
                            {/each}

                            <!-- App slots -->
                            {#each selected.apps as app, i}
                                <!-- svelte-ignore a11y-no-static-element-interactions -->
                                <div
                                    class="app-slot"
                                    class:dragging={dragging === app}
                                    style="
                                        left:{toPercent(app.x, COLS)}%;
                                        top:{toPercent(app.y, ROWS)}%;
                                        width:{toPercent(app.width, COLS)}%;
                                        height:{toPercent(app.height, ROWS)}%;
                                        --fill-mix:{[8, 12, 16, 6, 10][i % 5]}%;
                                        --border-mix:{[20, 30, 40, 15, 35][
                                        i % 5
                                    ]}%;
                                    "
                                    on:mousedown={(e) =>
                                        startDrag(e, app, "move")}
                                    on:dblclick|stopPropagation={() =>
                                        isBrowserApp(app) && openUrlModal(i)}
                                >
                                    <div class="slot-inner">
                                        {#if app.icon}
                                            <img
                                                class="slot-icon"
                                                src="data:image/png;base64,{app.icon}"
                                                alt={app.name}
                                            />
                                        {:else}
                                            <div class="slot-icon-fallback">
                                                {app.name
                                                    .charAt(0)
                                                    .toUpperCase()}
                                            </div>
                                        {/if}
                                    </div>

                                    {#if isBrowserApp(app)}
                                        <button
                                            class="url-badge"
                                            on:click|stopPropagation={() =>
                                                openUrlModal(i)}
                                            aria-label="Edit URLs"
                                        >
                                            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
                                                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
                                            </svg>
                                            {#if (app.urls?.length ?? 0) > 0}
                                                <span class="url-count">{app.urls!.length}</span>
                                            {/if}
                                        </button>
                                    {/if}

                                    <button
                                        class="slot-remove"
                                        on:click|stopPropagation={() =>
                                            removeApp(i)}
                                        aria-label="Remove {app.name}">×</button
                                    >

                                    <!-- Resize handles for all corners -->
                                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                                    <div
                                        class="resize-handle resize-nw"
                                        on:mousedown|stopPropagation={(e) =>
                                            startDrag(e, app, "resize", "nw")}
                                    ></div>
                                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                                    <div
                                        class="resize-handle resize-ne"
                                        on:mousedown|stopPropagation={(e) =>
                                            startDrag(e, app, "resize", "ne")}
                                    ></div>
                                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                                    <div
                                        class="resize-handle resize-sw"
                                        on:mousedown|stopPropagation={(e) =>
                                            startDrag(e, app, "resize", "sw")}
                                    ></div>
                                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                                    <div
                                        class="resize-handle resize-se"
                                        on:mousedown|stopPropagation={(e) =>
                                            startDrag(e, app, "resize", "se")}
                                    ></div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </div>
            {:else}
                <div class="no-selection">
                    Select a profile or create a new one
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

{#if showUrlModal}
    <BrowserUrlModal
        appName={urlModalAppName}
        urls={urlModalUrls}
        onsave={onUrlSave}
        onclose={() => (showUrlModal = false)}
    />
{/if}

<style>
    .workspaces-root {
        display: grid;
        grid-template-columns: 128px 1fr;
        min-height: 440px;
        border-radius: var(--radius-md);
        overflow: hidden;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-subtle);
    }

    /* Sidebar */
    .profile-list {
        border-right: 1px solid var(--color-border-subtle);
        display: flex;
        flex-direction: column;
        background: rgba(0, 0, 0, 0.16);
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
        gap: 5px;
        padding: 9px 12px;
        border: none;
        background: transparent;
        text-align: left;
        border-bottom: 1px solid var(--color-border-subtle);
        transition: background var(--transition-fast);
        position: relative;
    }
    .profile-row:hover {
        background: var(--color-button-bg);
    }
    .profile-row.active {
        background: transparent;
    }
    .profile-row.active::before {
        content: "";
        position: absolute;
        left: 0;
        top: 6px;
        bottom: 6px;
        width: 3px;
        border-radius: 0 3px 3px 0;
        background: var(--color-accent);
    }
    .profile-row.active .profile-name {
        color: var(--color-accent);
        font-weight: 600;
    }

    .profile-name {
        font-size: 12.5px;
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
        transition: all var(--transition-fast);
    }
    .new-profile-btn:hover {
        color: var(--color-text-secondary);
        border-color: var(--color-border-strong);
    }

    /* Editor */
    .profile-editor {
        display: flex;
        flex-direction: column;
    }

    .editor-header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 12px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .name-input {
        flex: 1;
        padding: 5px 9px;
        border-radius: var(--radius-sm);
        border: 1px solid transparent;
        background: transparent;
        color: var(--color-text-primary);
        font-size: 13px;
        font-family: inherit;
        font-weight: 500;
        transition: border-color var(--transition-fast);
    }
    .name-input:hover {
        border-color: var(--color-border-medium);
    }
    .name-input:focus {
        outline: none;
        border-color: var(--color-accent-border);
        background: var(--color-button-bg);
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 6px;
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
        transition: all var(--transition-fast);
    }
    .add-app-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .save-btn {
        padding: 5px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 12px;
        font-family: inherit;
        font-weight: 500;
        transition: all var(--transition-fast);
    }
    .save-btn:hover:not(:disabled) {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }
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
        transition: all var(--transition-fast);
    }
    .delete-btn:hover {
        background: var(--color-danger-soft);
        color: var(--color-danger);
    }

    /* Canvas */
    .canvas-wrap {
        padding: 16px;
        flex: 1;
        background: rgba(0, 0, 0, 0.08);
    }

    .canvas {
        position: relative;
        width: 100%;
        aspect-ratio: 16/9;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        overflow: hidden;
        user-select: none;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    }

    /* Grid cells — subtle dotted guide */
    .grid-cell {
        position: absolute;
        box-sizing: border-box;
        border-right: 1px solid var(--color-border-subtle);
        border-bottom: 1px solid var(--color-border-subtle);
        pointer-events: none;
        opacity: 0.5;
    }

    /* App slots */
    .app-slot {
        position: absolute;
        background: color-mix(
            in srgb,
            var(--color-accent) var(--fill-mix),
            transparent
        );
        border: 1px solid
            color-mix(
                in srgb,
                var(--color-accent) var(--border-mix),
                transparent
            );
        border-radius: var(--radius-sm);
        cursor: grab;
        box-sizing: border-box;
        display: flex;
        align-items: center;
        justify-content: center;
        transition:
            box-shadow 120ms ease,
            background 120ms ease;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
    }
    .app-slot:hover:not(.dragging) {
        background: color-mix(
            in srgb,
            var(--color-accent) calc(var(--fill-mix) + 8%),
            transparent
        );
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }
    .app-slot.dragging {
        cursor: grabbing;
        box-shadow: 0 8px 20px rgba(0, 0, 0, 0.25);
        z-index: 10;
    }
    .app-slot:active {
        cursor: grabbing;
    }

    .slot-inner {
        display: flex;
        align-items: center;
        justify-content: center;
        pointer-events: none;
    }

    .slot-icon {
        width: 24px;
        height: 24px;
        border-radius: 4px;
        object-fit: contain;
        opacity: 0.9;
    }

    .slot-icon-fallback {
        width: 24px;
        height: 24px;
        border-radius: 4px;
        background: color-mix(in srgb, var(--color-accent) 60%, transparent);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 11px;
        font-weight: 700;
        color: #000;
        opacity: 0.7;
    }

    .slot-remove {
        position: absolute;
        top: 3px;
        right: 3px;
        background: none;
        border: none;
        color: var(--color-accent);
        font-size: 16px;
        line-height: 1;
        padding: 2px 6px;
        opacity: 0;
        border-radius: 4px;
        transition: opacity 120ms ease;
    }
    .app-slot:hover .slot-remove {
        opacity: 0.7;
    }
    .slot-remove:hover {
        opacity: 1 !important;
    }

    .url-badge {
        position: absolute;
        bottom: 4px;
        left: 4px;
        display: inline-flex;
        align-items: center;
        gap: 3px;
        padding: 2px 5px;
        border: none;
        border-radius: 4px;
        background: var(--color-accent);
        color: #fff;
        font-size: 10px;
        cursor: pointer;
        opacity: 0;
        transition: opacity 120ms ease;
        z-index: 2;
    }
    .app-slot:hover .url-badge {
        opacity: 0.8;
    }
    .url-badge:hover {
        opacity: 1 !important;
    }

    .url-count {
        font-weight: 600;
        font-size: 9px;
    }

    .resize-handle {
        position: absolute;
        width: 12px;
        height: 12px;
        background: linear-gradient(
            135deg,
            transparent 50%,
            var(--color-accent) 50%
        );
        opacity: 0.6;
        transition: opacity 120ms ease;
    }
    .resize-handle:hover {
        opacity: 1;
    }

    .resize-nw {
        top: 0;
        left: 0;
        cursor: nw-resize;
        background: linear-gradient(
            135deg,
            var(--color-accent) 50%,
            transparent 50%
        );
        border-top-left-radius: 3px;
    }
    .resize-ne {
        top: 0;
        right: 0;
        cursor: ne-resize;
        background: linear-gradient(
            -135deg,
            var(--color-accent) 50%,
            transparent 50%
        );
        border-top-right-radius: 3px;
    }
    .resize-sw {
        bottom: 0;
        left: 0;
        cursor: sw-resize;
        background: linear-gradient(
            45deg,
            var(--color-accent) 50%,
            transparent 50%
        );
        border-bottom-left-radius: 3px;
    }
    .resize-se {
        bottom: 0;
        right: 0;
        cursor: se-resize;
        background: linear-gradient(
            -45deg,
            var(--color-accent) 50%,
            transparent 50%
        );
        border-bottom-right-radius: 3px;
    }

    .no-selection {
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 13px;
        color: var(--color-text-dim);
        padding: 32px;
        text-align: center;
    }

    /* Pro gate */
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
        transition: opacity var(--transition-fast);
    }
    .upgrade-btn:hover {
        opacity: 0.85;
    }

    .no-space-msg {
        font-size: 12px;
        color: var(--color-danger);
        animation: fade-in 150ms ease;
    }

    @keyframes fade-in {
        from {
            opacity: 0;
            transform: translateX(4px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    .gap-control {
        display: flex;
        align-items: center;
        gap: 6px;
        flex-shrink: 0;
    }

    .gap-label {
        font-size: 11px;
        color: var(--color-text-dim);
        white-space: nowrap;
    }

    .gap-slider {
        width: 64px;
        accent-color: var(--color-accent);
    }

    .gap-value {
        font-size: 11px;
        color: var(--color-text-dim);
        min-width: 24px;
    }
</style>
