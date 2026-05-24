<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
</script>

<PreferencesSection
    title="License"
    description="View license information and terms"
>
    <div class="license-info">
        <div class="license-item">
            <span class="license-label">License Type</span>
            <span class="license-value">MIT License</span>
        </div>
        <div class="license-item">
            <span class="license-label">Version</span>
            <span class="license-value">0.1.0</span>
        </div>
        <div class="license-item">
            <span class="license-label">Licensee</span>
            <span class="license-value">Personal Use</span>
        </div>
    </div>
    <div class="license-text">
        <p>
            Permission is hereby granted, free of charge, to any person obtaining a copy
            of this software and associated documentation files (the "Software"), to deal
            in the Software without restriction, including without limitation the rights
            to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
            copies of the Software.
        </p>
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

    .license-text {
        padding: 16px;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-subtle);
    }

    .license-text p {
        font-size: 12px;
        color: var(--color-text-secondary);
        line-height: 1.6;
        margin: 0;
    }
</style>
