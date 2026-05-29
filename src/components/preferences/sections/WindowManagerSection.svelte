<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";
    import { updateWindowManager } from "$services/preferences";
    import { preferences } from "$stores/preferences";

    $: opacity = $preferences?.window_manager.opacity ?? 80;
</script>

<PreferencesSection
    title="Window Manager"
    description="Configure window behavior and positioning"
>
    <PreferenceItem
        label="Snap to edges"
        description="Automatically snap window to screen edges when dragging"
    >
        <Toggle label="" id="snap-edges" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="Remember position"
        description="Save window position between sessions"
    >
        <Toggle label="" id="remember-position" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="Hide on focus loss"
        description="Automatically hide window when it loses focus"
    >
        <Toggle label="" id="hide-focus-loss" checked={true} />
    </PreferenceItem>

    <PreferenceItem
        label="Window opacity"
        description="Adjust the transparency of the window"
    >
        <div class="slider-wrap">
            <BrightnessSlider
                value={opacity}
                onchange={(v) => updateWindowManager(v)}
            />
        </div>
    </PreferenceItem>
</PreferencesSection>

<style>
    .slider-wrap {
        position: relative;
        width: 240px;
        height: 4px;
    }
</style>
