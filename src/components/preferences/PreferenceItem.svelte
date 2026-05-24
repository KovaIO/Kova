<script lang="ts">
    let { label, description, children } = $props();
</script>

<div class="item">
    <div class="item-info">
        <div class="item-label">{label}</div>
        {#if description}
            <div class="item-description">{description}</div>
        {/if}
    </div>
    <div class="item-control">
        {@render children()}
    </div>
</div>

<style>
    .item {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 20px;
        padding: 16px;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-subtle);
    }

    .item-info {
        flex: 1;
        min-width: 0;
    }

    .item-label {
        font-size: 14px;
        font-weight: 500;
        color: var(--color-text-primary);
        margin-bottom: 4px;
    }

    .item-description {
        font-size: 12px;
        color: var(--color-text-secondary);
        line-height: 1.4;
    }

    .item-control {
        flex-shrink: 0;
        display: flex;
        align-items: center;
    }
</style>
