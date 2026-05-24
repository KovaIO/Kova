<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
</script>

<PreferencesSection
    title="Help & Feedback"
    description="Get help or send feedback about the application"
>
    <div class="help-links">
        <button class="help-link">
            <div class="help-link-content">
                <div class="help-link-title">Documentation</div>
                <div class="help-link-description">View user guides and tutorials</div>
            </div>
        </button>
        <button class="help-link">
            <div class="help-link-content">
                <div class="help-link-title">Community Forum</div>
                <div class="help-link-description">Join discussions with other users</div>
            </div>
        </button>
        <button class="help-link">
            <div class="help-link-content">
                <div class="help-link-title">Report a Bug</div>
                <div class="help-link-description">Submit bug reports and issues</div>
            </div>
        </button>
        <button class="help-link">
            <div class="help-link-content">
                <div class="help-link-title">Feature Request</div>
                <div class="help-link-description">Suggest new features or improvements</div>
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
