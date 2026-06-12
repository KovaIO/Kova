<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import { openUrl } from "@tauri-apps/plugin-opener";
</script>

<PreferencesSection
    title="Help & Feedback"
    description="If you have any issues or feedback, feel free to reach out."
>
    <div class="help-links">
        <button
            class="help-link"
            onclick={() =>
                openUrl("mailto:campfire.softwaredev@gmail.com")}
        >
            <div class="help-link-content">
                <div class="help-link-title">Email</div>
                <div class="help-link-description">
                    campfire.softwaredev@gmail.com
                </div>
            </div>
        </button>
        <button
            class="help-link"
            onclick={() => openUrl("https://x.com/Kevin_rh0")}
        >
            <div class="help-link-content">
                <div class="help-link-title">X (Twitter)</div>
                <div class="help-link-description">https://x.com/Kevin_rh0</div>
            </div>
        </button>
    </div>
</PreferencesSection>

<style>
    .help-links {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .help-link {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 16px;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-subtle);
        text-decoration: none;
        cursor: pointer;
        transition: all 150ms ease;
    }

    .help-link:hover {
        background: var(--color-button-bg-hover);
        border-color: var(--color-border-medium);
    }

    .help-link-content {
        flex: 1;
    }

    .help-link-title {
        font-size: 14px;
        font-weight: 600;
        color: var(--color-text-primary);
        margin-bottom: 4px;
    }

    .help-link-description {
        font-size: 12px;
        color: var(--color-text-secondary);
    }
</style>
