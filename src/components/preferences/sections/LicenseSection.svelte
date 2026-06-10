<script lang="ts">
    import PreferencesSection from "../PreferencesSection.svelte";
    import { activateLicense, getPortalUrl, license } from "$stores/license";
    import { openUrl } from "@tauri-apps/plugin-opener";

    let email = "";
    let loadingPortal = false;

    $: isPro = $license?.tier === "pro";

    async function upgrade() {
        openUrl("https://appkova.com/#pricing");
    }

    async function activate() {
        if (!email.trim()) return;

        await activateLicense(email);
        email = "";
    }

    async function manageSubscription() {
        loadingPortal = true;
        try {
            const url = await getPortalUrl();
            openUrl(url);
        } catch (e) {
            console.error("failed to get portal URL:", e);
        } finally {
            loadingPortal = false;
        }
    }

    function formatTimestamp(ts?: number | null) {
        if (!ts) return "-";

        return new Date(ts * 1000).toLocaleString();
    }
</script>

<PreferencesSection
    title="License"
    description="View your plan and feature access"
>
    <div class="license-info">
        <div class="license-item">
            <span class="license-label">Plan</span>
            {#if isPro}
                <span class="license-value pro">Pro</span>
            {:else}
                <button class="link-button" on:click={upgrade}>
                    Upgrade to Pro
                </button>
            {/if}
        </div>
        {#if isPro}
            <div class="license-item">
                <span class="license-label">Email</span>
                <span class="license-value">
                    {$license?.email}
                </span>
            </div>

            <div class="license-item">
                <span class="license-label">Device ID</span>
                <span class="license-value">
                    {$license?.device_id}
                </span>
            </div>

            <div class="license-item">
                <span class="license-label">Activated</span>
                <span class="license-value">
                    {formatTimestamp($license?.activated_at)}
                </span>
            </div>

            <div class="license-item">
                <span class="license-label">Subscription</span>
                <button
                    class="portal-button"
                    on:click={manageSubscription}
                    disabled={loadingPortal}
                >
                    {loadingPortal ? "Loading..." : "Manage Subscription"}
                </button>
            </div>
        {:else}
            <div class="license-item">
                <div class="license-label">Already purchased?</div>
                <div class="activation">
                    <input
                        bind:value={email}
                        type="email"
                        placeholder="Enter purchase email"
                    />
                    <button class="activate-button" on:click={activate}>
                        Activate License
                    </button>
                </div>
            </div>
        {/if}
        <div class="license-item">
            <span class="license-label">Version</span>
            <span class="license-value">0.1.0</span>
        </div>
    </div>
</PreferencesSection>

<style>
    .license-info {
        display: flex;
        flex-direction: column;
        gap: 12px;
        margin-bottom: 20px;
    }

    .license-item {
        display: flex;
        justify-content: space-between;
        padding: 12px 16px;
        background: var(--color-surface-elevated);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border-subtle);
    }

    .license-label {
        font-size: 13px;
        font-weight: 500;
        color: var(--color-text-secondary);
    }

    .license-value {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-primary);
    }

    .license-value.pro {
        color: var(--color-accent);
    }

    .activation {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .activation input {
        width: 220px;
        padding: 6px 10px;
        background: transparent;
        border: 1px solid var(--color-border-subtle);
        border-radius: var(--radius-sm);
        color: var(--color-text-primary);
    }

    .link-button {
        border: none;
        background: none;
        color: var(--color-accent);
        cursor: pointer;
        font-size: 13px;
        font-weight: 600;
    }

    .activate-button {
        padding: 6px 12px;

        border: 1px solid var(--color-border-subtle);
        border-radius: var(--radius-sm);

        background: var(--color-accent);
        color: var(--color-accent-text);
    }

    .portal-button {
        padding: 6px 12px;
        font-size: 13px;
        font-weight: 600;

        border: 1px solid var(--color-border-subtle);
        border-radius: var(--radius-sm);

        background: transparent;
        color: var(--color-accent);
        cursor: pointer;
    }

    .portal-button:hover {
        background: var(--color-surface-elevated);
    }

    .portal-button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
