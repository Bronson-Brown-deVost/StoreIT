// TypeScript types mirroring crates/storeit-server/src/dto.rs

// -- Error --

export interface ErrorDetail {
  code: string;
  message: string;
}

export interface ErrorResponse {
  error: ErrorDetail;
}

// -- Auth --

export interface AuthModeResponse {
  mode: "oidc" | "local";
}

export interface UserResponse {
  id: string;
  email: string;
  display_name: string;
  is_admin: boolean;
}

export interface GroupResponse {
  id: string;
  name: string;
  role: string;
}

export interface MeResponse {
  user: UserResponse;
  groups: GroupResponse[];
  active_group_id: string;
}

export interface SwitchGroupRequest {
  group_id: string;
}

// -- Location --

export interface LocationResponse {
  id: string;
  group_id: string;
  parent_id: string | null;
  name: string;
  description: string | null;
  latitude: number | null;
  longitude: number | null;
  created_at: string;
  updated_at: string;
}

export interface CreateLocationRequest {
  parent_id?: string | null;
  name: string;
  description?: string | null;
  latitude?: number | null;
  longitude?: number | null;
}

export interface UpdateLocationRequest {
  name?: string | null;
  description?: string | null;
  latitude?: number | null;
  longitude?: number | null;
}

export interface LocationTreeNode {
  id: string;
  name: string;
  description: string | null;
  latitude: number | null;
  longitude: number | null;
  children: LocationTreeNode[];
}

// -- Container --

export interface ContainerResponse {
  id: string;
  group_id: string;
  parent_location_id: string | null;
  parent_container_id: string | null;
  name: string;
  description: string | null;
  color: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateContainerRequest {
  parent_type: string;
  parent_id: string;
  name: string;
  description?: string | null;
  color?: string | null;
}

export interface UpdateContainerRequest {
  name?: string | null;
  description?: string | null;
  color?: string | null;
}

export interface MoveRequest {
  target_type: string;
  target_id: string;
}

export interface AncestryNode {
  entity_type: string;
  id: string;
  name: string;
}

// -- Item --

export interface ItemResponse {
  id: string;
  group_id: string;
  container_id: string | null;
  location_id: string | null;
  name: string;
  description: string | null;
  aliases: string[];
  keywords: string[];
  category: string | null;
  barcode: string | null;
  material: string | null;
  color: string | null;
  condition_notes: string | null;
  quantity: number;
  created_at: string;
  updated_at: string;
}

export interface CreateItemRequest {
  parent_type: string;
  parent_id: string;
  name: string;
  description?: string | null;
  aliases?: string[] | null;
  keywords?: string[] | null;
  category?: string | null;
  barcode?: string | null;
  material?: string | null;
  color?: string | null;
  condition_notes?: string | null;
  quantity?: number | null;
}

export interface UpdateItemRequest {
  name?: string | null;
  description?: string | null;
  aliases?: string[] | null;
  keywords?: string[] | null;
  category?: string | null;
  barcode?: string | null;
  material?: string | null;
  color?: string | null;
  condition_notes?: string | null;
  quantity?: number | null;
}

// -- Photo --

export interface PhotoResponse {
  id: string;
  entity_type: string;
  entity_id: string;
  mime_type: string;
  is_primary: boolean;
  rotation: number;
  created_at: string;
}

// -- Search --

export interface SearchResultItem {
  entity_type: string;
  entity_id: string;
  score: number;
}

export interface SearchResponse {
  results: SearchResultItem[];
}

// -- NFC Tags --

export interface NfcTagResponse {
  id: string;
  group_id: string;
  tag_uri: string;
  entity_type: string | null;
  entity_id: string | null;
  created_at: string;
  assigned_at: string | null;
}

export interface CreateNfcTagRequest {
  tag_uri: string;
}

export interface AssignNfcTagRequest {
  entity_type: string;
  entity_id: string;
}

export interface NfcResolveResponse {
  tag_id: string;
  entity_type: string;
  entity_id: string;
  entity_name: string;
  location_path: string[];
}

export interface NfcUidResolveResponse {
  status: "assigned" | "unassigned" | "unknown";
  tag_id?: string;
  entity_type?: string;
  entity_id?: string;
  entity_name?: string;
  location_path?: string[];
}

export interface RegisterAndAssignNfcRequest {
  tag_uri: string;
  entity_type: string;
  entity_id: string;
}

// -- Local Auth --

export interface LocalLoginRequest {
  username: string;
  password: string;
}

// -- Admin --

export interface AdminUserResponse {
  id: string;
  username: string;
  email: string;
  display_name: string;
  is_admin: boolean;
  created_at: string;
}

export interface CreateLocalUserRequest {
  username: string;
  email: string;
  display_name: string;
  password: string;
  is_admin?: boolean;
}

export interface UpdateLocalUserRequest {
  email?: string;
  display_name?: string;
  is_admin?: boolean;
}

export interface ResetPasswordRequest {
  new_password: string;
}

export interface CreateGroupRequest {
  name: string;
}

export interface AdminGroupResponse {
  id: string;
  name: string;
  created_at: string;
}

export interface AddMemberRequest {
  user_id: string;
  role: string;
}

export interface GroupMemberResponse {
  user: AdminUserResponse;
  role: string;
}

// -- Admin Settings --

export interface AdminSettingsResponse {
  image_storage_path: string;
  image_storage_path_readonly: boolean;
}

export interface UpdateSettingsRequest {
  image_storage_path: string;
}

// -- Backup / Restore --

export interface BackupRequest {
  include_images: boolean;
}

export interface BackupJobResponse {
  job_id: string;
}

export interface JobStatusResponse {
  status: string;
  progress: number;
  total: number;
  error?: string;
}

export interface RestoreOptions {
  mode: string;
  image_storage_path?: string;
}

// -- AI Identification --

export interface IdentificationResponse {
  name: string;
  category: string | null;
  description: string | null;
  aliases: string[];
  keywords: string[];
  color: string | null;
  material: string | null;
  condition_notes: string | null;
}
