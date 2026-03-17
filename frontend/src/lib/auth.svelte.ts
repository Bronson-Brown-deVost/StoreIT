import {
  getMe,
  getAuthMode,
  localLogin as localLoginApi,
  switchGroup as switchGroupApi,
  logout as logoutApi,
  ApiClientError,
  type MeResponse,
  type GroupResponse,
  type AuthModeResponse,
} from "~/api";

class AuthStore {
  data = $state<MeResponse | undefined>(undefined);
  loading = $state(true);
  authMode = $state<"oidc" | "local" | undefined>(undefined);

  user = $derived(this.data?.user);
  groups = $derived<GroupResponse[]>(this.data?.groups ?? []);
  activeGroupId = $derived(this.data?.active_group_id);

  constructor() {
    this.init();
    window.addEventListener("storeit:unauthenticated", () => {
      this.data = undefined;
    });
  }

  private async init() {
    try {
      const [me, mode] = await Promise.all([
        getMe().catch((err) => {
          if (err instanceof ApiClientError && err.status === 401) return undefined;
          throw err;
        }),
        getAuthMode(),
      ]);
      this.data = me;
      this.authMode = mode?.mode;
    } catch {
      this.data = undefined;
    } finally {
      this.loading = false;
    }
  }

  login() {
    window.location.href = "/api/v1/auth/login";
  }

  async localLogin(username: string, password: string) {
    const result = await localLoginApi({ username, password });
    this.data = result;
  }

  async switchGroup(groupId: string) {
    const updated = await switchGroupApi({ group_id: groupId });
    this.data = updated;
  }

  async logout() {
    await logoutApi();
    this.data = undefined;
  }

  refetch() {
    this.init();
  }
}

export const auth = new AuthStore();
