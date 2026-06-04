<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import { license } from "$stores/license";

    $: tierLabel = $license?.tier === "pro" ? "Pro" : "Free";
</script>

<PreferencesSection
    title="License"
    description="View your plan and feature access"
>
    <div class="license-info">
        <div class="license-item">
            <span class="license-label">Plan</span>
            <span class="license-value" class:pro={$license?.tier === "pro"}>
                {tierLabel}
            </span>
        </div>
        <div class="license-item">
            <span class="license-label">Version</span>
            <span class="license-value">0.1.0</span>
        </div>
    </div>
</PreferencesSection>

<style>
    .license-info {
        display: flex;
        flex-direction: column;
        gap: 12px;
        margin-bottom: 20px;
    }

    .license-item {
        display: flex;
        justify-content: space-between;
        padding: 12px 16px;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-subtle);
    }

    .license-label {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-secondary);
    }

    .license-value {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
    }

    .license-value.pro {
        color: var(--color-accent);
    }
</style>
