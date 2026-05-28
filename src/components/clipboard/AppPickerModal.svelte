<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { createEventDispatcher, onMount } from "svelte";

    const dispatch = createEventDispatcher<{
        pick: { name: string; path: string; icon?: string };
        close: void;
    }>();

    interface Process {
        name: string;
        path: string;
        icon?: string;
    }

    let processes: Process[] = [];
    let search = "";
    let loading = true;
    let searchInput: HTMLInputElement;

    $: filtered = processes.filter((p) =>
        p.name.toLowerCase().includes(search.toLowerCase()),
    );

    onMount(async () => {
        try {
            processes = await invoke<Process[]>("get_running_processes");
        } finally {
            loading = false;
            searchInput?.focus();
        }
    });

    async function browseFile() {
        const selected = await open({
            title: "Select an application",
            filters: [
                { name: "Executable files", extensions: ["exe"] },
                { name: "All files", extensions: ["*"] },
            ],
            directory: false,
            multiple: false,
            defaultPath: "C:\\Program Files",
        });

        if (!selected) return;

        const path = selected as string;
        const name = path.split("\\").pop() ?? path;
        dispatch("pick", { name, path });
    }

    function pick(p: Process) {
        dispatch("pick", { name: p.name, path: p.path, icon: p.icon });
    }

    function handleBackdrop(e: MouseEvent) {
        if (e.target === e.currentTarget) dispatch("close");
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
    class="backdrop"
    on:click={handleBackdrop}
    on:keydown={(e) => e.key === "Escape" && dispatch("close")}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
>
    <div class="modal">
        <div class="modal-header">
            <span class="modal-title">Select an app to ignore</span>
            <button
                class="close-btn"
                aria-label="Close"
                on:click={() => dispatch("close")}
            >
                <svg
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                >
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
            </button>
        </div>

        <div class="search-wrap">
            <svg
                class="search-icon"
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
                class="search-input"
                type="text"
                placeholder="Search running apps..."
                bind:value={search}
                bind:this={searchInput}
            />
        </div>

        <div class="process-list">
            {#if loading}
                <div class="state-msg">Loading processes…</div>
            {:else if filtered.length === 0}
                <div class="state-msg">No results for "{search}"</div>
            {:else}
                {#each filtered as p}
                    <button
                        class="process-item"
                        aria-label="Select {p.name}"
                        on:click={() => pick(p)}
                    >
                        {#if p.icon}
                            <img
                                class="process-icon"
                                src={`data:image/png;base64,${p.icon}`}
                                alt=""
                            />
                        {:else}
                            <div class="process-icon placeholder"></div>
                        {/if}
                        <div class="process-info">
                            <span class="process-name">{p.name}</span>
                            <span class="process-path">{p.path}</span>
                        </div>
                    </button>
                {/each}
            {/if}
        </div>

        <div class="modal-footer">
            <button
                class="browse-btn"
                aria-label="Browse for application"
                on:click={browseFile}
            >
                <svg
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path
                        d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                    />
                </svg>
                Browse for app
            </button>
        </div>
    </div>
</div>

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
        backdrop-filter: blur(4px);
    }

    .modal {
        width: 420px;
        max-height: 480px;
        background: var(--color-surface-elevated);
        border: 1px solid var(--color-border-medium);
        border-radius: var(--radius-lg);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 16px 0;
        flex-shrink: 0;
    }

    .modal-title {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
    }

    .close-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 24px;
        height: 24px;
        border: none;
        background: transparent;
        color: var(--color-text-dim);
        border-radius: 4px;
        transition: all var(--transition-fast);
    }
    .close-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .search-wrap {
        position: relative;
        padding: 12px 16px;
        flex-shrink: 0;
    }

    .search-icon {
        position: absolute;
        left: 28px;
        top: 50%;
        transform: translateY(-50%);
        color: var(--color-text-dim);
        pointer-events: none;
    }

    .search-input {
        width: 100%;
        padding: 8px 12px 8px 32px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-primary);
        font-size: 13px;
        font-family: inherit;
        transition: border-color var(--transition-fast);
    }
    .search-input::placeholder {
        color: var(--color-text-dim);
    }
    .search-input:focus {
        outline: none;
        border-color: var(--color-accent-border);
    }

    .process-list {
        flex: 1;
        overflow-y: auto;
        scrollbar-width: none;
        padding: 0 8px;
    }
    .process-list::-webkit-scrollbar {
        display: none;
    }

    .state-msg {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 32px;
        font-size: 12px;
        color: var(--color-text-dim);
    }

    .process-item {
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        padding: 9px 10px;
        border: none;
        border-radius: var(--radius-sm);
        background: transparent;
        text-align: left;
        transition: background var(--transition-fast);
    }
    .process-item:hover {
        background: var(--color-button-bg-hover);
    }

    .process-icon {
        width: 24px;
        height: 24px;
        flex-shrink: 0;
        border-radius: 4px;
        object-fit: contain;
    }

    .process-icon.placeholder {
        background: var(--color-button-bg);
        border: 1px solid var(--color-border-subtle);
    }

    .process-info {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .process-name {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-secondary);
    }

    .process-path {
        font-size: 11px;
        color: var(--color-text-dim);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .modal-footer {
        padding: 12px 16px;
        border-top: 1px solid var(--color-border-subtle);
        flex-shrink: 0;
    }

    .browse-btn {
        display: inline-flex;
        align-items: center;
        gap: 7px;
        padding: 8px 14px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-secondary);
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        transition: all var(--transition-fast);
    }
    .browse-btn:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
        border-color: var(--color-border-strong);
    }
</style>
