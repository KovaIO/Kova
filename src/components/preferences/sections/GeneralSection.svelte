<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";
    import { updateGeneral } from "$services/preferences";
    import { canUse, license } from "$stores/license";
    import { preferences, refreshBrightness } from "$stores/preferences";
    import type { GeneralPreferences } from "$types/preferences";

    // const LANGUAGES = [
    //     { value: "en", label: "English" },
    //     { value: "es", label: "Spanish" },
    //     { value: "fr", label: "French" },
    //     { value: "de", label: "German" },
    // ] as const;

    $: general = $preferences?.general;
    $: monitorDim = general?.monitor_dim ?? 100;
    // $: language = general?.language ?? "en";
    $: monitorDimmingEnabled = canUse("monitor_dimming", $license);

    function onFocus() {
        refreshBrightness();
    }

    onMount(() => {
        window.addEventListener("focus", onFocus);
    });

    onDestroy(() => {
        window.removeEventListener("focus", onFocus);
    });

    // function onLanguageChange(event: Event) {
    //     const value = (event.target as HTMLSelectElement).value;
    //     updateGeneral({ language: value });
    // }
</script>

<PreferencesSection
    title="General"
    description="Configure general application settings"
>
    <PreferenceItem
        label="Launch at startup"
        description="Automatically start Kova when you log in to your computer"
    >
        <Toggle
            label=""
            id="launch-startup"
            checked={general?.launch_at_startup ?? false}
            onchange={(launch_at_startup) =>
                updateGeneral({ launch_at_startup })}
        />
    </PreferenceItem>

    <PreferenceItem
        label="Show in menu bar"
        description="Display Kova icon in the system menu bar"
    >
        <Toggle
            label=""
            id="show-menubar"
            checked={general?.show_menu_bar ?? true}
            onchange={(show_menu_bar) => updateGeneral({ show_menu_bar })}
        />
    </PreferenceItem>

    <div class="monitor-dim-group">
        <PreferenceItem
            label="Monitor dim"
            description="Adjust the background dim level when the monitor is open"
        >
            <div class="slider-wrap">
                <BrightnessSlider
                    value={monitorDim}
                    disabled={!monitorDimmingEnabled}
                    onchange={(v) => updateGeneral({ monitor_dim: v })}
                />
            </div>
        </PreferenceItem>

        {#if !monitorDimmingEnabled}
            <p class="tier-hint">Upgrade to Pro to adjust monitor dimming.</p>
        {/if}
    </div>

    <!-- <PreferenceItem
        label="Language"
        description="Select your preferred language for the interface"
    >
        <select class="select" value={language} on:change={onLanguageChange}>
            {#each LANGUAGES as option}
                <option value={option.value}>{option.label}</option>
            {/each}
        </select>
    </PreferenceItem> -->
</PreferencesSection>

<style>
    .monitor-dim-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .tier-hint {
        margin: 0;
        padding: 0 16px 4px;
        font-size: 12px;
        color: var(--color-accent);
        line-height: 1.4;
    }

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
