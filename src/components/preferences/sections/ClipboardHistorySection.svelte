<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
</script>

<PreferencesSection
    title="Clipboard History"
    description="Manage clipboard history and behavior"
>
    <PreferenceItem
        label="Enable clipboard history"
        description="Keep track of items copied to clipboard"
    >
        <Toggle label="" id="enable-clipboard" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="History limit"
        description="Maximum number of items to keep in history"
    >
        <select class="select">
            <option>10 items</option>
            <option selected>25 items</option>
            <option>50 items</option>
            <option>100 items</option>
            <option>Unlimited</option>
        </select>
    </PreferenceItem>

    <PreferenceItem
        label="Auto-clear on exit"
        description="Clear clipboard history when application closes"
    >
        <Toggle label="" id="auto-clear" />
    </PreferenceItem>

    <PreferenceItem
        label="Ignore passwords"
        description="Don't save password fields to clipboard history"
    >
        <Toggle label="" id="ignore-passwords" checked={true} />
    </PreferenceItem>
</PreferencesSection>

<style>
    .select {
        padding: 8px 12px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-medium);
        background: var(--color-button-bg);
        color: var(--color-text-primary);
        font-size: 13px;
        cursor: pointer;
        min-width: 140px;
    }

    .select:hover {
        background: var(--color-button-bg-hover);
    }

    .select:focus {
        outline: none;
        border-color: var(--color-accent);
    }
</style>
