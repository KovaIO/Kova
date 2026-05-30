<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import { updateWindowManager } from "$services/preferences";
    import { preferences } from "$stores/preferences";

    $: windowManager = $preferences?.window_manager;
</script>

<PreferencesSection
    title="Window Manager"
    description="Configure window behavior and layout"
>
    <PreferenceItem
        label="Auto-layout"
        description="Automatically arrange windows for optimal workspace usage"
    >
        <Toggle
            label=""
            id="auto-layout"
            checked={windowManager?.auto_layout ?? true}
            onchange={(auto_layout) => updateWindowManager({ auto_layout })}
        />
    </PreferenceItem>

    <PreferenceItem
        label="Window switcher"
        description="Enable quick switching between open windows"
    >
        <Toggle
            label=""
            id="window-switcher"
            checked={windowManager?.window_switcher ?? true}
            onchange={(window_switcher) =>
                updateWindowManager({ window_switcher })}
        />
    </PreferenceItem>
</PreferencesSection>
