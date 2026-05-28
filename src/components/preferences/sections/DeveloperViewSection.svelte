<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
</script>

<PreferencesSection
    title="Developer View"
    description="Advanced settings for developers and debugging"
>
    <PreferenceItem
        label="Enable debug mode"
        description="Show additional debug information in console"
    >
        <Toggle label="" id="debug-mode" />
    </PreferenceItem>

    <PreferenceItem
        label="Show dev tools"
        description="Display browser developer tools"
    >
        <Toggle label="" id="show-devtools" />
    </PreferenceItem>

    <PreferenceItem
        label="Log level"
        description="Set the verbosity of application logs"
    >
        <select class="select">
            <option>Error</option>
            <option>Warning</option>
            <option selected>Info</option>
            <option>Debug</option>
            <option>Trace</option>
        </select>
    </PreferenceItem>

    <PreferenceItem
        label="Enable hot reload"
        description="Automatically reload on file changes"
    >
        <Toggle label="" id="hot-reload" checked={true} />
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
