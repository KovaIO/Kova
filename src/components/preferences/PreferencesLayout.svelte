<script lang="ts">
    let { sidebar, content } = $props();
</script>

<div class="layout">
    <div class="sidebar">
        {@render sidebar()}
    </div>
    <div class="content">
        {@render content()}
    </div>
</div>

<style>
    .layout {
        display: flex;
        height: 100%;
        background: var(--color-main-bg);
    }

    .sidebar {
        width: 240px;
        flex-shrink: 0;
        border-right: 1px solid var(--color-border-subtle);
        padding: 20px 0;
        overflow-y: auto;
        overflow-x: hidden;
    }

    .content {
        flex: 1;
        overflow-y: auto;
        padding: 32px 40px;
    }
</style>
