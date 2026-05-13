// EGrab - Parser Interface Protocol (L5)
// Derived from: docs/protocols/parser-interface.md v1.0.0

import type { JsonObject, JsonValue, ProductData, ScrapeErrorInfo } from './data-models';

export interface PlatformParser {
  platform_id(): string;
  can_handle(url: string): boolean;
  extract_item_id(url: string): string;
  parse(page: PageHandle): Promise<ProductData>;
}

export interface PageHandle {
  url(): Promise<string>;
  title(): Promise<string>;
  evaluate(script: string): Promise<JsonValue>;
  content(): Promise<string>;
}

export interface PageContext {
  url: string;
  item_id: string;
  page_title: string;
  raw_evaluate_result: JsonValue;
  raw_html?: string;
}

export interface ParserConfig {
  keep_raw_html: boolean;
  image_url_cleaning: boolean;
}

export interface ParseResult {
  product: ProductData | null;
  raw_data: JsonObject;
  errors: ScrapeErrorInfo[];
}
