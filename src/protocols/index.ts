// EGrab - Protocol Index (L5)
// Central export for all protocol types

export type {
  KnownPlatform,
  Platform,
  TaskStatus,
  ImageType,
  ScrapeStep,
  TaskId,
  Iso8601String,
  JsonValue,
  JsonObject,
  ProtocolVersion,
  ProductData,
  ImageRef,
  SkuItem,
  PriceRange,
  ShopInfo,
  Description,
  SpecItem,
  Task,
  ImageRecord,
  TaskFilter,
  TaskSummary,
  TaskDetail,
  TaskResult,
  ScrapeErrorInfo,
  ErrorCode,
  IpcError,
  ConnectionInfo,
  ConnectionState,
  BrowserLaunchCommand,
  TabInfo,
  AppConfig,
  MetaJsonDocument,
  RawJsonDocument,
  ResizeDetail,
  ResizeResult,
} from './data-models';

export type {
  CdpConnectCommand,
  CdpDisconnectCommand,
  CdpStatusCommand,
  CdpListTabsCommand,
  CdpAutoConnectCommand,
  StartScrapeCommand,
  CancelScrapeCommand,
  GetTaskHistoryCommand,
  GetTaskDetailCommand,
  OpenFolderCommand,
  DeleteTaskCommand,
  GetConfigCommand,
  SetConfigCommand,
  ResizeImagesCommand,
  IpcResult,
  IpcCommand,
} from './ipc-commands';

export type {
  ScrapeProgressPayload,
  ScrapeCompletePayload,
  ScrapeErrorPayload,
  CdpStateChangedPayload,
  BackendEvent,
} from './events';

export type {
  PlatformParser,
  PageHandle,
  PageContext,
  ParserConfig,
  ParseResult,
} from './parser';

export type {
  StorageEngine,
  DuplicateTaskConflict,
  TaskUpdate,
  ImageIndexInput,
  ArchiveStructure,
  ArchiveEntry,
} from './storage';

export type { CdpManager, CdpEndpoint } from './cdp-manager';

export type {
  ImageDownloader,
  DownloadImageInput,
  DownloadOptions,
  DownloadImageResult,
  DownloadBatchResult,
} from './downloader';

export type { ScraperEngine, ScraperParseOutput } from './scraper-engine';

export type { ConfigManager } from './config';
