<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Select from "$components/Select.svelte";

    import { preferences } from "$stores/preferences";
    import { updateAppearance } from "$services/preferences";

    import type {
        AccentColor,
        WindowDensity,
        MetricCardStyle,
    } from "$types/preferences";
    import { ACCENT_COLORS } from "$utils/accent-colors";

    const DENSITIES: {
        value: WindowDensity;
        label: string;
    }[] = [
        {
            value: "normal",
            label: "Normal",
        },
        {
            value: "wide",
            label: "Wide",
        },
    ];

    const METRIC_STYLES: {
        value: MetricCardStyle;
        label: string;
    }[] = [
        {
            value: "block",
            label: "Block",
        },
        {
            value: "ring",
            label: "Ring",
        },
    ];

    $: appearance = $preferences?.appearance;

    $: accentColor = appearance?.accent_color ?? "purple";
    $: windowDensity = appearance?.window_density ?? "normal";
    $: metricCardStyle = appearance?.metric_card_style ?? "block";

    function selectAccentColor(color: AccentColor) {
        updateAppearance({
            accent_color: color,
        });
    }
</script>

<PreferencesSection
    title="Appearance"
    description="Customize the look and feel of the application"
>
    <PreferenceItem
        label="Accent color"
        description="Choose the primary accent color used throughout the interface"
    >
        <div class="color-picker">
            {#each ACCENT_COLORS as option}
                <button
                    type="button"
                    class:selected={accentColor === option.value}
                    class="color-option"
                    title={option.label}
                    on:click={() => selectAccentColor(option.value)}
                >
                    <span
                        class="color-preview"
                        style={`background:${option.color}`}
                    ></span>
                </button>
            {/each}
        </div>
    </PreferenceItem>

    <PreferenceItem
        label="Window density"
        description="Adjust how much horizontal space is used throughout the interface"
    >
        <Select
            bind:value={windowDensity}
            options={DENSITIES}
            onchange={(v) => updateAppearance({ window_density: v as WindowDensity })}
        />
    </PreferenceItem>

    <PreferenceItem
        label="Metric cards"
        description="Choose the visual style for system metric indicators"
    >
        <Select
            bind:value={metricCardStyle}
            options={METRIC_STYLES}
            onchange={(v) => updateAppearance({ metric_card_style: v as MetricCardStyle })}
        />
    </PreferenceItem>
</PreferencesSection>

<style>
    .color-picker {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .color-option {
        width: 34px;
        height: 34px;
        padding: 0;
        border-radius: 999px;
        border: 2px solid transparent;
        background: transparent;
        cursor: pointer;
        transition:
            transform 0.15s ease,
            border-color 0.15s ease;
    }

    .color-option:hover {
        transform: scale(1.05);
    }

    .color-option.selected {
        border-color: var(--color-accent);
    }

    .color-preview {
        display: block;
        width: 100%;
        height: 100%;
        border-radius: inherit;
        border: 1px solid var(--color-border-medium);
    }
</style>
