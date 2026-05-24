<script lang="ts">
    let { title, description, children } = $props();
</script>

<section class="section">
    <div class="section-header">
        <h3 class="section-title">{title}</h3>
        {#if description}
            <p class="section-description">{description}</p>
        {/if}
    </div>
    <div class="section-content">
        {@render children()}
    </div>
</section>

<style>
    .section {
        margin-bottom: 32px;
    }

    .section-header {
        margin-bottom: 20px;
        padding-bottom: 16px;
        border-bottom: 1px solid var(--color-border-subtle);
    }

    .section-title {
        font-size: 18px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin: 0 0 8px 0;
        letter-spacing: -0.01em;
    }

    .section-description {
        font-size: 13px;
        color: var(--color-text-secondary);
        margin: 0;
        line-height: 1.5;
    }

    .section-content {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }
</style>
