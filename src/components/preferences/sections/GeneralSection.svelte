<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
</script>

<PreferencesSection
    title="General"
    description="Configure general application settings"
>
    <PreferenceItem
        label="Launch at startup"
        description="Automatically start Kova when you log in to your computer"
    >
        <Toggle label="" id="launch-startup" />
    </PreferenceItem>

    <PreferenceItem
        label="Show in menu bar"
        description="Display Kova icon in the system menu bar"
    >
        <Toggle label="" id="show-menubar" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="Language"
        description="Select your preferred language for the interface"
    >
        <select class="select">
            <option>English</option>
            <option>Spanish</option>
            <option>French</option>
            <option>German</option>
        </select>
    </PreferenceItem>

    <PreferenceItem
        label="Theme"
        description="Choose between light and dark mode"
    >
        <select class="select">
            <option>Dark</option>
            <option>Light</option>
            <option>System</option>
        </select>
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
