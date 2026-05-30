<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import BrightnessSlider from "$components/BrightnessSlider.svelte";
    import { updateGeneral } from "$services/preferences";
    import { preferences } from "$stores/preferences";
    import type { GeneralPreferences } from "$types/preferences";

    const LANGUAGES = [
        { value: "en", label: "English" },
        { value: "es", label: "Spanish" },
        { value: "fr", label: "French" },
        { value: "de", label: "German" },
    ] as const;

    const THEMES: { value: GeneralPreferences["theme"]; label: string }[] = [
        { value: "dark", label: "Dark" },
        { value: "light", label: "Light" },
        { value: "system", label: "System" },
    ];

    $: general = $preferences?.general;
    $: monitorDim = general?.monitor_dim ?? 90;
    $: language = general?.language ?? "en";
    $: theme = general?.theme ?? "system";

    function onLanguageChange(event: Event) {
        const value = (event.target as HTMLSelectElement).value;
        updateGeneral({ language: value });
    }

    function onThemeChange(event: Event) {
        const value = (event.target as HTMLSelectElement)
            .value as GeneralPreferences["theme"];
        updateGeneral({ theme: value });
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
            onchange={(launch_at_startup) => updateGeneral({ launch_at_startup })}
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
        <select class="select" value={language} on:change={onLanguageChange}>
            {#each LANGUAGES as option}
                <option value={option.value}>{option.label}</option>
            {/each}
        </select>
    </PreferenceItem>

    <PreferenceItem
        label="Theme"
        description="Choose between light and dark mode"
    >
        <select class="select" value={theme} on:change={onThemeChange}>
            {#each THEMES as option}
                <option value={option.value}>{option.label}</option>
            {/each}
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
