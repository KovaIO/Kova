<script lang="ts">
    let { sections, activeSection, onSectionChange } = $props();

    type Section = {
        id: string;
        label: string;
    };
</script>

<nav class="sidebar">
    <div class="sidebar-header">
        <h2>Preferences</h2>
    </div>
    <ul class="section-list">
        {#each sections as section}
            <li>
                <button
                    class="section-button"
                    class:active={activeSection === section.id}
                    onclick={() => onSectionChange?.(section.id)}
                >
                    <span class="label">{section.label}</span>
                </button>
            </li>
        {/each}
    </ul>
</nav>

<style>
    .sidebar {
        height: 100%;
        display: flex;
        flex-direction: column;
        overflow-x: hidden;
    }

    .sidebar-header {
        padding: 0 20px 16px;
        border-bottom: 1px solid var(--color-border-subtle);
        margin-bottom: 8px;
    }

    .sidebar-header h2 {
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.02em;
        color: var(--color-text-primary);
        margin: 0;
    }

    .section-list {
        list-style: none;
        margin: 0;
        padding: 0;
    }

    .section-button {
        width: 100%;
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 20px;
        border: none;
        background: transparent;
        color: var(--color-text-secondary);
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 150ms ease;
        text-align: left;
        border-radius: 6px;
        margin: 2px 12px;
    }

    .section-button:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .section-button.active {
        background: var(--color-accent-soft);
        color: var(--color-accent);
    }

    .label {
        flex: 1;
    }
</style>
