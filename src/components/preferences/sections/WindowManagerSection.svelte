<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import PreferenceItem from "../PreferenceItem.svelte";
    import Toggle from "$components/Toggle.svelte";
    import { updateWindowManager } from "$services/preferences";
    import { canUse, license } from "$stores/license";
    import { preferences } from "$stores/preferences";

    $: windowManager = $preferences?.window_manager;
    $: windowManagerEnabled = windowManager?.enabled ?? true;
    $: autoLayoutEnabled = canUse("auto_layout", $license);
    $: windowSwitcherEnabled = canUse("window_switcher", $license);
</script>

<PreferencesSection
    title="Window Manager"
    description="Configure window behavior and layout"
>
    <PreferenceItem
        label="Enable window manager"
        description="Turn window management features on or off"
    >
        <Toggle
            label=""
            id="enable-window-manager"
            checked={windowManagerEnabled}
            onchange={(enabled) => updateWindowManager({ enabled })}
        />
    </PreferenceItem>

    <div class="feature-group">
        <PreferenceItem
            label="Auto-layout"
            description="Automatically arrange windows for optimal workspace usage"
        >
            <Toggle
                label=""
                id="auto-layout"
                checked={windowManager?.auto_layout ?? false}
                disabled={!windowManagerEnabled || !autoLayoutEnabled}
                onchange={(auto_layout) => updateWindowManager({ auto_layout })}
            />
        </PreferenceItem>

        {#if !autoLayoutEnabled}
            <p class="tier-hint">Upgrade to Pro to enable auto-layout.</p>
        {/if}
    </div>

    <div class="feature-group">
        <PreferenceItem
            label="Window switcher"
            description="Enable quick switching between open windows"
        >
            <Toggle
                label=""
                id="window-switcher"
                checked={windowManager?.window_switcher ?? false}
                disabled={!windowManagerEnabled || !windowSwitcherEnabled}
                onchange={(window_switcher) =>
                    updateWindowManager({ window_switcher })}
            />
        </PreferenceItem>

        {#if !windowSwitcherEnabled}
            <p class="tier-hint">Upgrade to Pro to enable window switcher.</p>
        {/if}
    </div>
</PreferencesSection>

<style>
    .feature-group {
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
</style>
