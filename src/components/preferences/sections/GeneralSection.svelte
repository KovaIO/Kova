<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { platform } from "@tauri-apps/plugin-os";
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";
    import { updateGeneral } from "$services/preferences";
    import { preferences, refreshBrightness } from "$stores/preferences";

    const isMacos = platform() === "macos";

    $: general = $preferences?.general;
    $: monitorDim = general?.monitor_dim ?? 100;

    function onFocus() {
        refreshBrightness();
    }

    onMount(() => {
        window.addEventListener("focus", onFocus);
    });

    onDestroy(() => {
        window.removeEventListener("focus", onFocus);
    });

    async function onMenuBarToggle(show_menu_bar: boolean) {
        await updateGeneral({ show_menu_bar });
        await invoke("set_menu_bar_visible", { visible: show_menu_bar });
    }
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

    {#if isMacos}
        <PreferenceItem
            label="Show in menu bar"
            description="Display Kova icon in the system menu bar"
        >
            <Toggle
                label=""
                id="show-menubar"
                checked={general?.show_menu_bar ?? true}
                onchange={onMenuBarToggle}
            />
        </PreferenceItem>
    {/if}

    <div class="monitor-dim-group">
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

    .slider-wrap {
        position: relative;
        width: 240px;
        height: 4px;
    }
</style>
