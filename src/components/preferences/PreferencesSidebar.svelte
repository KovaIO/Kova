<script lang="ts">
    let { sections, activeSection, onSectionChange } = $props();

    type Section = {
        id: string;
        label: string;
    };
</script>

<nav class="sidebar">
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

    .section-list {
        list-style: none;
        margin: 0;
        padding: 4px 10px;
    }

    .section-button {
        width: 100%;
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 12px;
        border: none;
        background: transparent;
        color: var(--color-text-secondary);
        font-size: 13px;
        font-weight: 500;
        transition: all 150ms ease;
        text-align: left;
        border-radius: 6px;
        margin: 0;
    }

    .section-button:hover {
        background: var(--color-button-bg-hover);
        color: var(--color-text-primary);
    }

    .section-button.active {
        background: var(--color-accent-soft);
        color: var(--color-accent);
        font-weight: 600;
    }

    .label {
        flex: 1;
    }
</style>
