<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { Component } from "svelte";
    import { preferences } from "$stores/preferences";

    export let icon: Component;
    export let label: string;
    export let value: number;
    export let displayValue: string;
    export let show: boolean = true;

    export let tab: string;

    let hovered = false;

    $: metricStyle = $preferences?.appearance.metric_card_style ?? "block";

    async function openMonitor() {
        await invoke("open_monitor", { tab });
    }
</script>

{#if show}
    <button
        class="card"
        class:ring={metricStyle === "ring"}
        aria-label="{label} metric"
        on:click={openMonitor}
        on:mouseenter={() => (hovered = true)}
        on:mouseleave={() => (hovered = false)}
    >
        {#if metricStyle === "ring"}
            <svg class="ring-svg" viewBox="0 0 36 36">
                <circle
                    class="ring-bg"
                    cx="18"
                    cy="18"
                    r="16.5"
                    fill="none"
                    stroke-width="3"
                />
                <circle
                    class="ring-fill"
                    cx="18"
                    cy="18"
                    r="16.5"
                    fill="none"
                    stroke-width="3"
                    stroke-dasharray="{value} {100 - value}"
                    stroke-dashoffset="25"
                />
            </svg>
        {:else}
            <div class="fill" style="height: {value}%;"></div>
        {/if}

        <div class="content normal">
            <div class="icon-wrap" class:hidden={hovered}>
                <svelte:component this={icon} size={18} strokeWidth={1.5} />
            </div>

            <div class="value-wrap" class:visible={hovered}>
                <span class="val">{displayValue}</span>
            </div>

            <span class="label">{label}</span>
        </div>

        {#if metricStyle !== "ring"}
            <div
                class="content accent"
                style="clip-path: inset({100 - value}% 0 0 0)"
            >
                <div class="icon-wrap" class:hidden={hovered}>
                    <svelte:component this={icon} size={18} strokeWidth={1.5} />
                </div>

                <div class="value-wrap" class:visible={hovered}>
                    <span class="val">{displayValue}</span>
                </div>

                <span class="label">{label}</span>
            </div>
        {/if}
    </button>
{/if}

<style>
    .card {
        position: relative;
        width: 80px;
        height: 84px;
        border-radius: 12px;
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border: 1px solid var(--color-border-subtle);
        overflow: hidden;
        flex-shrink: 0;
    }

    .card.ring {
        background: transparent;
        border: none;
    }

    .card.ring .label {
        bottom: 20px;
        font-size: 9px;
    }

    .card.ring .icon-wrap {
        top: 40%;
    }

    .fill {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        background: linear-gradient(
            to top,
            var(--color-accent),
            var(--color-accent-hover)
        );
        transition: height 600ms cubic-bezier(0.4, 0, 0.2, 1);
        border-radius: 0 0 12px 12px;
        opacity: 0.9;
    }

    .ring-svg {
        position: absolute;
        inset: 0px;
    }

    .ring-bg {
        stroke: var(--color-border-subtle);
        opacity: 0.4;
    }

    .ring-fill {
        stroke: var(--color-accent);
        stroke-linecap: round;
        transition: stroke-dasharray 600ms cubic-bezier(0.4, 0, 0.2, 1);
    }

    .content {
        position: relative;
        z-index: 1;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 0 0 6px 0;
        gap: 0;
    }

    .content.accent {
        position: absolute;
        inset: 0;
        z-index: 2;
        pointer-events: none;
    }

    .content.accent .icon-wrap,
    .content.accent .val,
    .content.accent .label {
        color: var(--color-accent-text);
    }

    .icon-wrap,
    .value-wrap {
        position: absolute;
        top: 50%;
        transform: translateY(-60%);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: var(--transition-slow);
    }

    .icon-wrap {
        opacity: 1;
        transform: translateY(-60%) scale(1);
    }

    .icon-wrap.hidden {
        opacity: 0;
        transform: translateY(-60%) scale(0.85);
    }

    .value-wrap {
        opacity: 0;
        transform: translateY(-60%) scale(0.9);
    }

    .value-wrap.visible {
        opacity: 1;
        transform: translateY(-60%) scale(1);
    }

    .icon-wrap {
        opacity: 1;
        transform: translateY(-60%) scale(1);
        color: var(--color-text-primary);
        opacity: 0.85;
    }

    .val {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
        letter-spacing: -0.02em;
        white-space: nowrap;
    }

    .label {
        position: absolute;
        bottom: 9px;
        font-size: 10px;
        font-weight: 500;
        letter-spacing: 0.04em;
        color: var(--color-text-secondary);
        text-transform: uppercase;
    }
</style>
