<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";
    import { updateGeneral } from "$services/preferences";
    import { preferences } from "$stores/preferences";

    $: general = $preferences?.general;
    $: monitorDim = general?.monitor_dim ?? 90;
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
        label="Monitor dim"
        description="Adjust the background dim level when the monitor is open"
    >
        <div class="slider-wrap">
            <BrightnessSlider
                value={monitorDim}
                onchange={(v) => updateGeneral({ monitor_dim: v })}
            />
        </div>
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
    .slider-wrap {
        position: relative;
        width: 240px;
        height: 4px;
    }

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
