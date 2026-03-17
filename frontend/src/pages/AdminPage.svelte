<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { auth } from "~/lib/auth.svelte";
  import {
    listAdminUsers,
    createAdminUser,
    deleteAdminUser,
    resetPassword,
    listAdminGroups,
    createAdminGroup,
    deleteAdminGroup,
    listGroupMembers,
    addGroupMember,
    removeGroupMember,
    getAdminSettings,
    updateAdminSettings,
    startBackup,
    getBackupStatus,
    backupDownloadUrl,
    startRestore,
    getRestoreStatus,
    type AdminUserResponse,
    type AdminGroupResponse,
    type GroupMemberResponse,
    type JobStatusResponse,
    ApiClientError,
  } from "~/api";

  const hasWebNfc = "NDEFReader" in window;

  // ---- Settings Section ----
  let settingsData = $state<any>(null);
  let settingsPath = $state("");
  let settingsSaving = $state(false);
  let settingsSuccess = $state<string | null>(null);
  let settingsError = $state<string | null>(null);
  let settingsInitialized = $state(false);

  async function loadSettings() {
    try {
      settingsData = await getAdminSettings();
      if (!settingsInitialized) {
        settingsPath = settingsData.image_storage_path;
        settingsInitialized = true;
      }
    } catch {
      // ignore
    }
  }

  async function handleSaveSettings(e: Event) {
    e.preventDefault();
    settingsError = null;
    settingsSuccess = null;
    settingsSaving = true;
    try {
      const updated = await updateAdminSettings({ image_storage_path: settingsPath });
      settingsPath = updated.image_storage_path;
      settingsSuccess = "Settings saved successfully.";
      await loadSettings();
    } catch (err) {
      settingsError = err instanceof ApiClientError ? err.message : "Failed to save settings";
    } finally {
      settingsSaving = false;
    }
  }

  // ---- Users Section ----
  let users = $state<AdminUserResponse[]>([]);
  let showCreateUser = $state(false);
  let newUsername = $state("");
  let newEmail = $state("");
  let newDisplayName = $state("");
  let newPassword = $state("");
  let newIsAdmin = $state(false);
  let userError = $state<string | null>(null);
  let resetUserId = $state<string | null>(null);
  let resetNewPassword = $state("");

  async function loadUsers() {
    try {
      users = await listAdminUsers();
    } catch {
      // ignore
    }
  }

  async function handleCreateUser(e: Event) {
    e.preventDefault();
    userError = null;
    try {
      await createAdminUser({
        username: newUsername,
        email: newEmail,
        display_name: newDisplayName,
        password: newPassword,
        is_admin: newIsAdmin,
      });
      showCreateUser = false;
      newUsername = "";
      newEmail = "";
      newDisplayName = "";
      newPassword = "";
      newIsAdmin = false;
      await loadUsers();
    } catch (err) {
      userError = err instanceof ApiClientError ? err.message : "Failed to create user";
    }
  }

  async function handleDeleteUser(user: AdminUserResponse) {
    if (!confirm(`Delete user "${user.username}"?`)) return;
    try {
      await deleteAdminUser(user.id);
      await loadUsers();
    } catch (err) {
      alert(err instanceof ApiClientError ? err.message : "Failed to delete user");
    }
  }

  async function handleResetPassword(e: Event) {
    e.preventDefault();
    if (!resetUserId) return;
    try {
      await resetPassword(resetUserId, { new_password: resetNewPassword });
      resetUserId = null;
      resetNewPassword = "";
    } catch (err) {
      alert(err instanceof ApiClientError ? err.message : "Failed to reset password");
    }
  }

  // ---- Groups Section ----
  let groups = $state<AdminGroupResponse[]>([]);
  let newGroupName = $state("");
  let selectedGroup = $state<string | null>(null);
  let members = $state<GroupMemberResponse[]>([]);
  let addUserId = $state("");
  let addRole = $state("member");

  async function loadGroups() {
    try {
      groups = await listAdminGroups();
    } catch {
      // ignore
    }
  }

  async function handleCreateGroup(e: Event) {
    e.preventDefault();
    if (!newGroupName) return;
    try {
      await createAdminGroup({ name: newGroupName });
      newGroupName = "";
      await loadGroups();
    } catch (err) {
      alert(err instanceof ApiClientError ? err.message : "Failed to create group");
    }
  }

  async function handleDeleteGroup(group: AdminGroupResponse) {
    if (!confirm(`Delete group "${group.name}"?`)) return;
    try {
      await deleteAdminGroup(group.id);
      if (selectedGroup === group.id) {
        selectedGroup = null;
        members = [];
      }
      await loadGroups();
    } catch (err) {
      alert(err instanceof ApiClientError ? err.message : "Failed to delete group");
    }
  }

  async function loadMembers(groupId: string) {
    selectedGroup = groupId;
    members = await listGroupMembers(groupId);
  }

  async function handleAddMember(e: Event) {
    e.preventDefault();
    if (!selectedGroup || !addUserId) return;
    try {
      await addGroupMember(selectedGroup, { user_id: addUserId, role: addRole });
      addUserId = "";
      await loadMembers(selectedGroup);
    } catch (err) {
      alert(err instanceof ApiClientError ? err.message : "Failed to add member");
    }
  }

  async function handleRemoveMember(userId: string) {
    if (!selectedGroup) return;
    try {
      await removeGroupMember(selectedGroup, userId);
      await loadMembers(selectedGroup);
    } catch (err) {
      alert(err instanceof ApiClientError ? err.message : "Failed to remove member");
    }
  }

  // ---- Backup Section ----
  let backupIncludeImages = $state(false);
  let backupJobId = $state<string | null>(null);
  let backupStatus = $state<JobStatusResponse | null>(null);
  let backupError = $state<string | null>(null);
  let backupPolling = $state(false);
  let backupPollTimer: ReturnType<typeof setInterval> | undefined;

  let backupProgress = $derived(() => {
    if (!backupStatus || backupStatus.total === 0) return 0;
    return Math.round((backupStatus.progress / backupStatus.total) * 100);
  });

  async function handleStartBackup() {
    backupError = null;
    backupStatus = null;
    backupJobId = null;
    try {
      const res = await startBackup({ include_images: backupIncludeImages });
      backupJobId = res.job_id;
      backupPolling = true;
      backupPollTimer = setInterval(async () => {
        try {
          const s = await getBackupStatus(res.job_id);
          backupStatus = s;
          if (s.status === "complete" || s.status === "failed") {
            clearInterval(backupPollTimer);
            backupPolling = false;
            if (s.status === "failed") backupError = s.error || "Backup failed";
          }
        } catch {
          clearInterval(backupPollTimer);
          backupPolling = false;
          backupError = "Failed to check backup status";
        }
      }, 500);
    } catch (err) {
      backupError = err instanceof ApiClientError ? err.message : "Failed to start backup";
    }
  }

  // ---- Restore Section ----
  let restoreFile = $state<File | null>(null);
  let restoreMode = $state("replace");
  let restoreImagePath = $state("");
  let restoreStatus = $state<JobStatusResponse | null>(null);
  let restoreError = $state<string | null>(null);
  let restoreSuccess = $state(false);
  let restorePolling = $state(false);
  let restorePollTimer: ReturnType<typeof setInterval> | undefined;
  let restoreSettingsLoaded = $state(false);

  let restoreProgress = $derived(() => {
    if (!restoreStatus || restoreStatus.total === 0) return 0;
    return Math.round((restoreStatus.progress / restoreStatus.total) * 100);
  });

  async function loadRestoreSettings() {
    try {
      const s = await getAdminSettings();
      if (!restoreImagePath) restoreImagePath = s.image_storage_path;
      restoreSettingsLoaded = true;
    } catch {
      // ignore
    }
  }

  async function handleStartRestore() {
    if (!restoreFile) return;

    if (restoreMode === "replace") {
      if (!confirm("This will permanently delete ALL existing data and replace it with the backup. Continue?")) return;
    }

    restoreError = null;
    restoreSuccess = false;
    restoreStatus = null;

    try {
      const options = {
        mode: restoreMode,
        image_storage_path: restoreImagePath || undefined,
      };
      const res = await startRestore(restoreFile, options);
      restorePolling = true;
      restorePollTimer = setInterval(async () => {
        try {
          const s = await getRestoreStatus(res.job_id);
          restoreStatus = s;
          if (s.status === "complete" || s.status === "failed") {
            clearInterval(restorePollTimer);
            restorePolling = false;
            if (s.status === "failed") restoreError = s.error || "Restore failed";
            if (s.status === "complete") restoreSuccess = true;
          }
        } catch {
          clearInterval(restorePollTimer);
          restorePolling = false;
          restoreError = "Failed to check restore status";
        }
      }, 500);
    } catch (err) {
      restoreError = err instanceof ApiClientError ? err.message : "Failed to start restore";
    }
  }

  // ---- NFC Provisioning Section ----
  let nfcBaseUrl = $state(`${window.location.origin}/nfc/tag?uid=`);
  let nfcProvisioning = $state(false);
  let nfcCount = $state(0);
  let nfcLastUid = $state<string | null>(null);
  let nfcError = $state<string | null>(null);
  let nfcStatus = $state<string | null>(null);
  let nfcAbortController: AbortController | null = null;

  async function startNfcProvisioning() {
    nfcError = null;
    nfcStatus = null;
    nfcCount = 0;
    nfcLastUid = null;
    nfcProvisioning = true;

    try {
      const ndef = new (window as any).NDEFReader();
      nfcAbortController = new AbortController();

      await ndef.scan({ signal: nfcAbortController.signal });
      nfcStatus = "Ready -- tap an NFC tag...";

      ndef.addEventListener(
        "reading",
        async (event: any) => {
          const uid: string = event.serialNumber?.replace(/:/g, "").toUpperCase() || "";
          if (!uid) {
            nfcError = "Could not read tag UID";
            return;
          }

          nfcStatus = `Writing to tag ${uid}...`;

          try {
            const url = `${nfcBaseUrl}${uid}`;
            await ndef.write(
              { records: [{ recordType: "url", data: url }] },
              { signal: nfcAbortController!.signal }
            );

            nfcLastUid = uid;
            nfcCount += 1;
            nfcStatus = `Tag ${uid} provisioned! Tap next tag...`;

            if (navigator.vibrate) navigator.vibrate(100);
          } catch (writeErr: any) {
            if (writeErr.name !== "AbortError") {
              nfcError = `Failed to write tag: ${writeErr.message}`;
            }
          }
        },
        { signal: nfcAbortController.signal }
      );
    } catch (err: any) {
      if (err.name === "NotAllowedError") {
        nfcError = "NFC permission denied. Grant NFC access and try again.";
      } else if (err.name === "NotSupportedError") {
        nfcError = "Web NFC is not supported on this device/browser.";
      } else {
        nfcError = `NFC error: ${err.message}`;
      }
      nfcProvisioning = false;
    }
  }

  function stopNfcProvisioning() {
    nfcAbortController?.abort();
    nfcAbortController = null;
    nfcProvisioning = false;
    nfcStatus = nfcCount > 0 ? `Done -- ${nfcCount} tags provisioned.` : null;
  }

  // ---- Lifecycle ----
  onMount(() => {
    if (auth.user?.is_admin) {
      loadSettings();
      loadUsers();
      loadGroups();
      loadRestoreSettings();
    }
  });

  onDestroy(() => {
    if (backupPollTimer) clearInterval(backupPollTimer);
    if (restorePollTimer) clearInterval(restorePollTimer);
    nfcAbortController?.abort();
  });
</script>

{#if !auth.user?.is_admin}
  <div class="p-6 text-center text-text-secondary">
    <h1 class="text-2xl font-bold mb-2">Access Denied</h1>
    <p>You need admin privileges to access this page.</p>
  </div>
{:else}
  <div class="max-w-2xl mx-auto p-4 space-y-8">
    <h1 class="text-2xl font-bold">Admin</h1>

    <!-- Settings Section -->
    <section>
      <h2 class="text-xl font-semibold mb-4">Settings</h2>

      {#if settingsData}
        <form onsubmit={handleSaveSettings} class="bg-surface-raised rounded-lg p-4 flex flex-col gap-3">
          <label class="text-sm text-text-secondary">Image Storage Path
          <input
            type="text"
            value={settingsPath}
            oninput={(e) => { settingsPath = e.currentTarget.value; settingsSuccess = null; }}
            disabled={settingsData.image_storage_path_readonly}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          />
          </label>
          {#if settingsData.image_storage_path_readonly}
            <p class="text-text-muted text-xs">
              Set via STOREIT_IMAGE_PATH environment variable (read-only)
            </p>
          {:else}
            <p class="text-text-muted text-xs">
              Changing this path only affects new images. Existing images remain at the previous location.
            </p>
            <button
              type="submit"
              disabled={settingsSaving}
              class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm self-start disabled:opacity-50"
            >
              {settingsSaving ? "Saving..." : "Save"}
            </button>
          {/if}
          {#if settingsError}
            <div class="text-red-400 text-sm">{settingsError}</div>
          {/if}
          {#if settingsSuccess}
            <div class="text-green-400 text-sm">{settingsSuccess}</div>
          {/if}
        </form>
      {:else}
        <p class="text-text-muted text-sm">Loading...</p>
      {/if}
    </section>

    <!-- NFC Provisioning Section -->
    {#if hasWebNfc}
      <section>
        <h2 class="text-xl font-semibold mb-4">NFC Tag Provisioning</h2>
        <div class="bg-surface-raised rounded-lg p-4 flex flex-col gap-3">
          <p class="text-sm text-text-secondary">
            Write NFC URL records to blank tags so they work with StoreIT on any phone
            (including iPhone). Each tag gets a unique URL containing its hardware UID.
          </p>

          <label class="text-sm text-text-secondary mt-1">Base URL
          <input
            type="text"
            value={nfcBaseUrl}
            oninput={(e) => nfcBaseUrl = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
            disabled={nfcProvisioning}
          />
          </label>

          <div class="flex items-center gap-3">
            {#if nfcProvisioning}
              <button
                onclick={stopNfcProvisioning}
                class="px-4 py-2 bg-red-700 hover:bg-red-600 text-white rounded-lg text-sm min-h-[44px]"
              >
                Stop
              </button>
            {:else}
              <button
                onclick={startNfcProvisioning}
                class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm min-h-[44px]"
              >
                Start Provisioning
              </button>
            {/if}

            {#if nfcCount > 0}
              <span class="text-sm text-text-secondary">
                {nfcCount} tag{nfcCount !== 1 ? "s" : ""} provisioned
              </span>
            {/if}
          </div>

          {#if nfcStatus}
            <p class="text-sm text-text-secondary">{nfcStatus}</p>
          {/if}

          {#if nfcLastUid}
            <p class="text-xs text-text-muted">
              Last: <code>{nfcLastUid}</code>
            </p>
          {/if}

          {#if nfcError}
            <p class="text-red-400 text-sm">{nfcError}</p>
          {/if}
        </div>
      </section>
    {/if}

    <!-- Backup Section -->
    <section>
      <h2 class="text-xl font-semibold mb-4">Backup</h2>
      <div class="bg-surface-raised rounded-lg p-4 flex flex-col gap-3">
        <div class="flex gap-4">
          <label class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
            <input
              type="radio"
              name="backup-mode"
              checked={!backupIncludeImages}
              onchange={() => backupIncludeImages = false}
            />
            Data only
          </label>
          <label class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
            <input
              type="radio"
              name="backup-mode"
              checked={backupIncludeImages}
              onchange={() => backupIncludeImages = true}
            />
            Data + Images
          </label>
        </div>

        <button
          onclick={handleStartBackup}
          disabled={backupPolling}
          class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm self-start disabled:opacity-50"
        >
          {backupPolling ? "Backing up..." : "Start Backup"}
        </button>

        {#if backupStatus && (backupPolling || backupStatus.status === "complete")}
          <div class="w-full bg-surface rounded-full h-3 overflow-hidden">
            <div
              class="bg-primary h-full transition-all duration-300"
              style:width="{backupProgress()}%"
            ></div>
          </div>
          <p class="text-text-muted text-xs">
            {backupStatus.progress} / {backupStatus.total} -- {backupProgress()}%
          </p>
        {/if}

        {#if backupStatus?.status === "complete" && backupJobId}
          <a
            href={backupDownloadUrl(backupJobId)}
            class="px-4 py-2 bg-green-700 hover:bg-green-600 text-white rounded-lg text-sm self-start"
            download=""
          >
            Download Backup
          </a>
        {/if}

        {#if backupError}
          <div class="text-red-400 text-sm">{backupError}</div>
        {/if}
      </div>
    </section>

    <!-- Restore Section -->
    <section>
      <h2 class="text-xl font-semibold mb-4">Restore</h2>
      <div class="bg-surface-raised rounded-lg p-4 flex flex-col gap-3">
        <label class="text-sm text-text-secondary">Backup file (.tar.gz)
        <input
          type="file"
          accept=".tar.gz,.tgz"
          onchange={(e) => restoreFile = e.currentTarget.files?.[0] ?? null}
          class="text-sm text-text-primary file:mr-3 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-primary file:text-white file:text-sm file:cursor-pointer"
        />
        </label>

        <div class="flex gap-4 mt-2">
          <label class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
            <input
              type="radio"
              name="restore-mode"
              checked={restoreMode === "replace"}
              onchange={() => restoreMode = "replace"}
            />
            Full Replace
          </label>
          <label class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
            <input
              type="radio"
              name="restore-mode"
              checked={restoreMode === "merge"}
              onchange={() => restoreMode = "merge"}
            />
            Merge
          </label>
        </div>
        <p class="text-text-muted text-xs">
          {restoreMode === "replace"
            ? "Wipe all existing data and replace with backup contents."
            : "Add backup data alongside existing data (new UUIDs, no dedup)."}
        </p>

        {#if restoreSettingsLoaded}
          <label class="text-sm text-text-secondary mt-2">Image Storage Path
          <input
            type="text"
            value={restoreImagePath}
            oninput={(e) => restoreImagePath = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
          />
          </label>
          <p class="text-text-muted text-xs">
            Set image storage path before restore if the backup includes images.
          </p>
        {/if}

        <button
          onclick={handleStartRestore}
          disabled={restorePolling || !restoreFile}
          class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm self-start disabled:opacity-50"
        >
          {restorePolling ? "Restoring..." : "Start Restore"}
        </button>

        {#if restoreStatus && (restorePolling || restoreStatus.status === "complete")}
          <div class="w-full bg-surface rounded-full h-3 overflow-hidden">
            <div
              class="bg-primary h-full transition-all duration-300"
              style:width="{restoreProgress()}%"
            ></div>
          </div>
          <p class="text-text-muted text-xs">
            {restoreStatus.progress} / {restoreStatus.total} -- {restoreProgress()}%
          </p>
        {/if}

        {#if restoreError}
          <div class="text-red-400 text-sm">{restoreError}</div>
        {/if}
        {#if restoreSuccess}
          <div class="text-green-400 text-sm">Restore completed successfully.</div>
        {/if}
      </div>
    </section>

    <!-- Users Section -->
    <section>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold">Users</h2>
        <button
          onclick={() => showCreateUser = !showCreateUser}
          class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm min-h-[36px]"
        >
          {showCreateUser ? "Cancel" : "Create User"}
        </button>
      </div>

      {#if showCreateUser}
        <form onsubmit={handleCreateUser} class="bg-surface-raised rounded-lg p-4 mb-4 flex flex-col gap-3">
          {#if userError}
            <div class="text-red-400 text-sm">{userError}</div>
          {/if}
          <input
            type="text"
            placeholder="Username"
            value={newUsername}
            oninput={(e) => newUsername = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
            required
          />
          <input
            type="email"
            placeholder="Email"
            value={newEmail}
            oninput={(e) => newEmail = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
            required
          />
          <input
            type="text"
            placeholder="Display Name"
            value={newDisplayName}
            oninput={(e) => newDisplayName = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
            required
          />
          <input
            type="password"
            placeholder="Password"
            value={newPassword}
            oninput={(e) => newPassword = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
            required
          />
          <label class="flex items-center gap-2 text-sm text-text-secondary">
            <input
              type="checkbox"
              checked={newIsAdmin}
              onchange={(e) => newIsAdmin = e.currentTarget.checked}
            />
            Admin
          </label>
          <button
            type="submit"
            class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm"
          >
            Create
          </button>
        </form>
      {/if}

      {#if resetUserId}
        <form onsubmit={handleResetPassword} class="bg-surface-raised rounded-lg p-4 mb-4 flex gap-2">
          <input
            type="password"
            placeholder="New password"
            value={resetNewPassword}
            oninput={(e) => resetNewPassword = e.currentTarget.value}
            class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm flex-1"
            required
          />
          <button
            type="submit"
            class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm"
          >
            Reset
          </button>
          <button
            type="button"
            onclick={() => resetUserId = null}
            class="px-4 py-2 bg-surface border border-border text-text-secondary rounded-lg text-sm"
          >
            Cancel
          </button>
        </form>
      {/if}

      <div class="space-y-2">
        {#each users as user (user.id)}
          <div class="bg-surface-raised rounded-lg p-4 flex items-center justify-between">
            <div>
              <div class="font-medium text-text-primary">
                {user.display_name}
                {#if user.is_admin}
                  <span class="ml-2 px-2 py-0.5 bg-primary/20 text-primary text-xs rounded">
                    Admin
                  </span>
                {/if}
              </div>
              <div class="text-sm text-text-muted">@{user.username} &middot; {user.email}</div>
            </div>
            <div class="flex gap-2">
              <button
                onclick={() => resetUserId = user.id}
                class="px-3 py-1.5 text-xs bg-surface border border-border text-text-secondary rounded hover:bg-surface-raised"
              >
                Reset Password
              </button>
              <button
                onclick={() => handleDeleteUser(user)}
                class="px-3 py-1.5 text-xs bg-red-900/30 border border-red-700 text-red-300 rounded hover:bg-red-900/50"
              >
                Delete
              </button>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- Groups Section -->
    <section>
      <h2 class="text-xl font-semibold mb-4">Groups</h2>

      <form onsubmit={handleCreateGroup} class="flex gap-2 mb-4">
        <input
          type="text"
          placeholder="New group name"
          value={newGroupName}
          oninput={(e) => newGroupName = e.currentTarget.value}
          class="bg-surface-raised border border-border rounded-lg px-3 py-2 text-text-primary text-sm flex-1"
          required
        />
        <button
          type="submit"
          class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm"
        >
          Create
        </button>
      </form>

      <div class="space-y-2 mb-6">
        {#each groups as group (group.id)}
          <div class="bg-surface-raised rounded-lg p-4 flex items-center justify-between">
            <button
              onclick={() => loadMembers(group.id)}
              class={`font-medium text-left ${selectedGroup === group.id ? "text-primary" : "text-text-primary"}`}
            >
              {group.name}
            </button>
            <button
              onclick={() => handleDeleteGroup(group)}
              class="px-3 py-1.5 text-xs bg-red-900/30 border border-red-700 text-red-300 rounded hover:bg-red-900/50"
            >
              Delete
            </button>
          </div>
        {/each}
      </div>

      {#if selectedGroup}
        <div class="bg-surface-raised rounded-lg p-4">
          <h3 class="font-semibold mb-3">
            Members of "{groups.find((g) => g.id === selectedGroup)?.name}"
          </h3>

          <form onsubmit={handleAddMember} class="flex gap-2 mb-4">
            <input
              type="text"
              placeholder="User ID"
              value={addUserId}
              oninput={(e) => addUserId = e.currentTarget.value}
              class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm flex-1"
              required
            />
            <select
              value={addRole}
              onchange={(e) => addRole = e.currentTarget.value}
              class="bg-surface border border-border rounded-lg px-3 py-2 text-text-primary text-sm"
            >
              <option value="member">Member</option>
              <option value="owner">Owner</option>
            </select>
            <button
              type="submit"
              class="px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm"
            >
              Add
            </button>
          </form>

          <div class="space-y-2">
            {#each members as member (member.user.id)}
              <div class="flex items-center justify-between py-2 border-b border-border last:border-0">
                <div>
                  <span class="text-text-primary">{member.user.display_name}</span>
                  <span class="text-text-muted text-sm ml-2">({member.role})</span>
                </div>
                <button
                  onclick={() => handleRemoveMember(member.user.id)}
                  class="px-3 py-1 text-xs text-red-300 hover:text-red-200"
                >
                  Remove
                </button>
              </div>
            {/each}
            {#if members.length === 0}
              <p class="text-text-muted text-sm">No members</p>
            {/if}
          </div>
        </div>
      {/if}
    </section>
  </div>
{/if}
