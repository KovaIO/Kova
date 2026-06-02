<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import Titlebar from "$components/TitleBar.svelte";
    import PreferencesLayout from "$components/preferences/PreferencesLayout.svelte";
    import PreferencesSidebar from "$components/preferences/PreferencesSidebar.svelte";
    import GeneralSection from "$components/preferences/sections/GeneralSection.svelte";
    import LicenseSection from "$components/preferences/sections/LicenseSection.svelte";
    import WindowManagerSection from "$components/preferences/sections/WindowManagerSection.svelte";
    import ClipboardHistorySection from "$components/preferences/sections/ClipboardHistorySection.svelte";
    import KeyboardShortcutsSection from "$components/preferences/sections/KeyboardShortcutsSection.svelte";
    import PowerManagementSection from "$components/preferences/sections/PowerManagementSection.svelte";
    import HelpFeedbackSection from "$components/preferences/sections/HelpFeedbackSection.svelte";
    import DeveloperViewSection from "$components/preferences/sections/DeveloperViewSection.svelte";

    let activeSection = $state("general");

    const sections = [
        { id: "general", label: "General" },
        { id: "license", label: "License" },
        { id: "window-manager", label: "Window Manager" },
        { id: "clipboard-history", label: "Clipboard History" },
        { id: "keyboard-shortcuts", label: "Keyboard Shortcuts" },
        { id: "power-management", label: "Power Management" },
        { id: "help-feedback", label: "Help & Feedback" },
        { id: "developer-view", label: "Developer View" },
    ];

    function handleSectionChange(sectionId: string) {
        activeSection = sectionId;
    }

    onMount(() => {
        const win = getCurrentWindow();
        const unlisten = win.onCloseRequested((event) => {
            event.preventDefault();
            win.hide();
        });

        return () => {
            unlisten.then((f) => f());
        };
    });
</script>

<div class="window">
    <Titlebar />
    <div class="content">
        {#snippet sidebar()}
            <PreferencesSidebar
                {sections}
                {activeSection}
                onSectionChange={handleSectionChange}
            />
        {/snippet}
        {#snippet content()}
            {#if activeSection === "general"}
                <GeneralSection />
            {:else if activeSection === "license"}
                <LicenseSection />
            {:else if activeSection === "window-manager"}
                <WindowManagerSection />
            {:else if activeSection === "clipboard-history"}
                <ClipboardHistorySection />
            {:else if activeSection === "keyboard-shortcuts"}
                <KeyboardShortcutsSection />
            {:else if activeSection === "power-management"}
                <PowerManagementSection />
            {:else if activeSection === "help-feedback"}
                <HelpFeedbackSection />
            {:else if activeSection === "developer-view"}
                <DeveloperViewSection />
            {/if}
        {/snippet}
        <PreferencesLayout {sidebar} {content} />
    </div>
</div>

<style>
    .window {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background: var(--color-main-bg);
        backdrop-filter: blur(var(--blur-glass));
        border-radius: var(--radius-md);
        border: 2px solid var(--color-border-subtle);
        overflow: hidden;
        color: white;
    }

    .content {
        flex: 1;
        overflow: hidden;
    }
</style>
