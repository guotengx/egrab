// EGrab - Storage Interface Protocol (L5)
// Derived from: docs/protocols/storage-interface.md v1.0.0

import type {
  ImageType,
  JsonObject,
  MetaJsonDocument,
  Platform,
  ProductData,
  RawJsonDocument,
  ScrapeErrorInfo,
  Task,
  TaskDetail,
  TaskFilter,
  TaskId,
  TaskStatus,
  TaskSummary,
} from './data-models';

export interface StorageEngine {
  init(): Promise<void>;
  create_task(url: string, platform: Platform, item_id: string, force?: boolean): Promise<Task>;
  update_task(task_id: TaskId, updates: TaskUpdate): Promise<void>;
  save_meta(task_id: TaskId, product: ProductData): Promise<string>;
  build_meta_document(task_id: TaskId, product: ProductData): Promise<MetaJsonDocument>;
  save_raw(task_id: TaskId, raw_data: JsonObject, parser_errors: ScrapeErrorInfo[]): Promise<string>;
  build_raw_document(
    task_id: TaskId,
    raw_data: JsonObject,
    parser_errors: ScrapeErrorInfo[],
  ): Promise<RawJsonDocument>;
  index_image(image: ImageIndexInput): Promise<void>;
  query_tasks(filter: TaskFilter): Promise<TaskSummary[]>;
  get_task_detail(task_id: TaskId): Promise<TaskDetail>;
  check_duplicate(platform: Platform, item_id: string): Promise<TaskId | null>;
  open_folder(path: string): Promise<boolean>;
}

export interface DuplicateTaskConflict {
  existing_task_id: TaskId;
  existing_folder_path: string | null;
  code: 'DUPLICATE_TASK';
}

export interface TaskUpdate {
  status?: TaskStatus;
  title?: string;
  folder_path?: string | null;
}

export interface ImageIndexInput {
  task_id: TaskId;
  type: ImageType;
  original_url: string;
  local_path: string | null;
  width: number | null;
  height: number | null;
  size_bytes: number | null;
}

export interface ArchiveStructure {
  folder_name: string;
  absolute_path: string;
  entries: ArchiveEntry[];
}

export interface ArchiveEntry {
  name: string;
  type: 'file' | 'directory';
  children?: ArchiveEntry[];
}
