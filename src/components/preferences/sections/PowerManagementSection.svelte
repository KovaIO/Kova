<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
</script>

<PreferencesSection
    title="Power Management"
    description="Configure power and battery settings"
>
    <PreferenceItem
        label="Reduce CPU usage when idle"
        description="Lower CPU consumption when application is not in use"
    >
        <Toggle label="" id="reduce-cpu" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="Disable animations on battery"
        description="Turn off animations when running on battery power"
    >
        <Toggle label="" id="disable-animations-battery" />
    </PreferenceItem>

    <PreferenceItem
        label="Idle timeout"
        description="Time before entering low-power mode"
    >
        <select class="select">
            <option>1 minute</option>
            <option selected>5 minutes</option>
            <option>10 minutes</option>
            <option>30 minutes</option>
            <option>Never</option>
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
