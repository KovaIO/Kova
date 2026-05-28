<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";

    let opacity = 90;
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
        <div class="slider-wrap" style="--thumb-position: {opacity}%">
            <input
                type="range"
                min="0"
                max="100"
                bind:value={opacity}
                aria-label="Window opacity"
            />
            <div class="track-fill" style="width: {opacity}%"></div>
        </div>
    </PreferenceItem>
</PreferencesSection>

<style>
    .slider-wrap {
        position: relative;
        width: 140px;
        height: 4px;
    }

    /* Track base */
    .slider-wrap::before {
        content: "";
        position: absolute;
        inset: 0;
        background: var(--color-track-base);
        border-radius: 999px;
        border: 1px solid var(--color-border-subtle);
    }

    /* Filled portion */
    .track-fill {
        position: absolute;
        top: 0;
        left: 0;
        bottom: 0;
        background: linear-gradient(
            to right,
            var(--color-accent),
            var(--color-accent-hover)
        );
        border-radius: 999px;
        pointer-events: none;
        transition: width 60ms linear;
        opacity: 0.9;
    }

    /* Thumb */
    .slider-wrap::after {
        content: "";
        position: absolute;
        top: 50%;
        left: calc(var(--thumb-position) - 6px);
        transform: translateY(-50%);
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: var(--color-input);
        pointer-events: none;
        transition: left 60ms linear;
        z-index: 1;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }

    /* Invisible native input on top */
    input[type="range"] {
        position: absolute;
        inset: 0;
        width: 100%;
        opacity: 0;
        z-index: 2;
        margin: 0;
        height: 20px;
        top: 50%;
        transform: translateY(-50%);
    }
</style>
