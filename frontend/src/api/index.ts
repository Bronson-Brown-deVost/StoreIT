import { get, post, put, del, postMultipart, fileUrl } from "./client";
import type {
  AuthModeResponse,
  MeResponse,
  LocalLoginRequest,
  SwitchGroupRequest,
  AdminUserResponse,
  CreateLocalUserRequest,
  ResetPasswordRequest,
  AdminGroupResponse,
  CreateGroupRequest,
  AddMemberRequest,
  GroupMemberResponse,
  AdminSettingsResponse,
  UpdateSettingsRequest,
  BackupRequest,
  BackupJobResponse,
  JobStatusResponse,
  RestoreOptions,
  LocationResponse,
  LocationTreeNode,
  CreateLocationRequest,
  UpdateLocationRequest,
  ContainerResponse,
  CreateContainerRequest,
  UpdateContainerRequest,
  MoveRequest,
  AncestryNode,
  ItemResponse,
  CreateItemRequest,
  UpdateItemRequest,
  PhotoResponse,
  SearchResponse,
  IdentificationResponse,
  NfcTagResponse,
  CreateNfcTagRequest,
  AssignNfcTagRequest,
  NfcResolveResponse,
  NfcUidResolveResponse,
  RegisterAndAssignNfcRequest,
} from "./types";

export type * from "./types";
export { ApiClientError } from "./client";

// -- Auth --

export const getAuthMode = () => get<AuthModeResponse>("/auth/mode");

export const getMe = () => get<MeResponse>("/auth/me");

export const localLogin = (req: LocalLoginRequest) =>
  post<MeResponse>("/auth/local/login", req);

export const switchGroup = (req: SwitchGroupRequest) =>
  put<MeResponse>("/auth/me/active-group", req);

export const logout = () => post<void>("/auth/logout");

// -- Admin --

export const listAdminUsers = () => get<AdminUserResponse[]>("/admin/users");

export const createAdminUser = (req: CreateLocalUserRequest) =>
  post<AdminUserResponse>("/admin/users", req);

export const deleteAdminUser = (id: string) => del(`/admin/users/${id}`);

export const resetPassword = (id: string, req: ResetPasswordRequest) =>
  put<void>(`/admin/users/${id}/password`, req);

export const listAdminGroups = () => get<AdminGroupResponse[]>("/admin/groups");

export const createAdminGroup = (req: CreateGroupRequest) =>
  post<AdminGroupResponse>("/admin/groups", req);

export const deleteAdminGroup = (id: string) => del(`/admin/groups/${id}`);

export const listGroupMembers = (groupId: string) =>
  get<GroupMemberResponse[]>(`/admin/groups/${groupId}/members`);

export const addGroupMember = (groupId: string, req: AddMemberRequest) =>
  post<void>(`/admin/groups/${groupId}/members`, req);

export const removeGroupMember = (groupId: string, userId: string) =>
  del(`/admin/groups/${groupId}/members/${userId}`);

export const getAdminSettings = () =>
  get<AdminSettingsResponse>("/admin/settings");

export const updateAdminSettings = (req: UpdateSettingsRequest) =>
  put<AdminSettingsResponse>("/admin/settings", req);

// -- Backup / Restore --

export const startBackup = (req: BackupRequest) =>
  post<BackupJobResponse>("/admin/backup", req);

export const getBackupStatus = (jobId: string) =>
  get<JobStatusResponse>(`/admin/backup/${jobId}/status`);

export const backupDownloadUrl = (jobId: string) =>
  fileUrl(`/admin/backup/${jobId}/download`);

export const startRestore = (file: File, options: RestoreOptions) => {
  const fd = new FormData();
  fd.append("file", file);
  fd.append("options", JSON.stringify(options));
  return postMultipart<BackupJobResponse>("/admin/restore", fd);
};

export const getRestoreStatus = (jobId: string) =>
  get<JobStatusResponse>(`/admin/restore/${jobId}/status`);

// -- Locations --

export const getLocations = () => get<LocationResponse[]>("/locations");

export const getLocationTree = () => get<LocationTreeNode[]>("/locations/tree");

export const getLocation = (id: string) =>
  get<LocationResponse>(`/locations/${id}`);

export const createLocation = (req: CreateLocationRequest) =>
  post<LocationResponse>("/locations", req);

export const updateLocation = (id: string, req: UpdateLocationRequest) =>
  put<LocationResponse>(`/locations/${id}`, req);

export const getLocationChildren = (id: string) =>
  get<LocationResponse[]>(`/locations/${id}/children`);

export const getLocationContainers = (id: string) =>
  get<ContainerResponse[]>(`/locations/${id}/containers`);

export const getLocationItems = (id: string) =>
  get<ItemResponse[]>(`/locations/${id}/items`);

// -- Containers --

export const listContainers = () => get<ContainerResponse[]>("/containers");

export const getContainer = (id: string) =>
  get<ContainerResponse>(`/containers/${id}`);

export const createContainer = (req: CreateContainerRequest) =>
  post<ContainerResponse>("/containers", req);

export const updateContainer = (id: string, req: UpdateContainerRequest) =>
  put<ContainerResponse>(`/containers/${id}`, req);

export const deleteContainer = (id: string) => del(`/containers/${id}`);

export const moveContainer = (id: string, req: MoveRequest) =>
  post<void>(`/containers/${id}/move`, req);

export const getContainerAncestry = (id: string) =>
  get<AncestryNode[]>(`/containers/${id}/ancestry`);

export const getContainerContainers = (id: string) =>
  get<ContainerResponse[]>(`/containers/${id}/containers`);

export const getContainerItems = (id: string) =>
  get<ItemResponse[]>(`/containers/${id}/items`);

// -- Items --

export const listItems = () => get<ItemResponse[]>("/items");

export const getItem = (id: string) => get<ItemResponse>(`/items/${id}`);

export const createItem = (req: CreateItemRequest) =>
  post<ItemResponse>("/items", req);

export const updateItem = (id: string, req: UpdateItemRequest) =>
  put<ItemResponse>(`/items/${id}`, req);

export const deleteItem = (id: string) => del(`/items/${id}`);

export const batchCreateItems = (reqs: CreateItemRequest[]) =>
  post<ItemResponse[]>("/items/batch", reqs);

export const moveItem = (id: string, req: MoveRequest) =>
  post<void>(`/items/${id}/move`, req);

// -- Photos --

export const uploadPhoto = (
  entityType: string,
  entityId: string,
  file: File,
) => {
  const fd = new FormData();
  fd.append("entity_type", entityType);
  fd.append("entity_id", entityId);
  fd.append("file", file);
  return postMultipart<PhotoResponse>("/photos", fd);
};

export const getEntityPhotos = (entityType: string, entityId: string) =>
  get<PhotoResponse[]>(
    `/photos/by-entity?entity_type=${entityType}&entity_id=${entityId}`,
  );

export const photoFileUrl = (id: string) => fileUrl(`/photos/${id}/file`);

export const photoThumbnailUrl = (id: string) => fileUrl(`/photos/${id}/thumbnail`);

export const deletePhoto = (id: string) => del(`/photos/${id}`);

export const rotatePhoto = (id: string, degrees: number) =>
  post<PhotoResponse>(`/photos/${id}/rotate`, { degrees });

// -- Search --

export const search = (q: string, limit?: number) => {
  const params = new URLSearchParams({ q });
  if (limit) params.set("limit", String(limit));
  return get<SearchResponse>(`/search?${params}`);
};

// -- Identify --

export const identifyPhoto = (file: File) => {
  const fd = new FormData();
  fd.append("photo", file);
  return postMultipart<IdentificationResponse>("/identify", fd);
};

export const identifyCorrect = (file: File, correction: string) => {
  const fd = new FormData();
  fd.append("photo", file);
  fd.append("correction", correction);
  return postMultipart<IdentificationResponse>("/identify/correct", fd);
};

// -- NFC Tags --

export const listNfcTags = () => get<NfcTagResponse[]>("/nfc-tags");

export const createNfcTag = (req: CreateNfcTagRequest) =>
  post<NfcTagResponse>("/nfc-tags", req);

export const getNfcTag = (id: string) =>
  get<NfcTagResponse>(`/nfc-tags/${id}`);

export const resolveNfcTag = (tagUri: string) =>
  get<NfcResolveResponse>(`/nfc-tags/resolve/${encodeURIComponent(tagUri)}`);

export const assignNfcTag = (id: string, req: AssignNfcTagRequest) =>
  put<NfcTagResponse>(`/nfc-tags/${id}/assign`, req);

export const unassignNfcTag = (id: string) =>
  put<void>(`/nfc-tags/${id}/unassign`, {});

export const deleteNfcTag = (id: string) => del(`/nfc-tags/${id}`);

export const resolveNfcTagByUid = (uid: string) =>
  get<NfcUidResolveResponse>(`/nfc-tags/resolve-uid?uid=${encodeURIComponent(uid)}`);

export const registerAndAssignNfcTag = (req: RegisterAndAssignNfcRequest) =>
  post<NfcTagResponse>("/nfc-tags/register-and-assign", req);
