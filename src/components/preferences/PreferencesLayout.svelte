<script lang="ts">
    import { onMount, tick } from "svelte";

    let { sidebar, content } = $props();

    let contentEl: HTMLElement;
    let canScrollMore = $state(false);

    function checkScroll() {
        if (!contentEl) return;
        const { scrollTop, scrollHeight, clientHeight } = contentEl;
        canScrollMore = scrollTop + clientHeight < scrollHeight - 8;
    }

    onMount(async () => {
        await tick();
        checkScroll();
    });
</script>

<div class="layout">
    <div class="sidebar">
        {@render sidebar()}
    </div>
    <div class="content-wrapper">
        <div
            class="content"
            bind:this={contentEl}
            onscroll={checkScroll}
        >
            {@render content()}
        </div>
        <div class="scroll-indicator" class:visible={canScrollMore}></div>
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
        scrollbar-width: none;
    }
    .sidebar::-webkit-scrollbar { display: none; }

    .content-wrapper {
        flex: 1;
        position: relative;
        overflow: hidden;
    }

    .content {
        height: 100%;
        overflow-y: auto;
        padding: 32px 40px;
        scrollbar-width: none;
        -ms-overflow-style: none;
    }
    .content::-webkit-scrollbar { display: none; }

    .scroll-indicator {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 72px;
        background: linear-gradient(to top, var(--color-main-bg) 20%, transparent);
        pointer-events: none;
        opacity: 0;
        transition: opacity 400ms ease;
    }

    .scroll-indicator.visible {
        opacity: 1;
    }
</style>