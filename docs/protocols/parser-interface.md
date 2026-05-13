# Protocol: Parser Interface

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-08
- 依赖的真相源：`docs/PRD.md` 1.0.0（L1）、`docs/ARCHITECTURE.md` 1.0.0（L2）第4.3节、`docs/protocols/data-models.md`

## 类型定义

```ts
import type { JsonObject, JsonValue, ProductData, ScrapeErrorInfo } from './data-models';

/** 平台解析器必须实现的统一接口。Rust 中为 trait，TypeScript 中为 interface。 */
export interface PlatformParser {
  /** 返回平台标识，如 "taobao"、"tmall"、"jd"。 */
  platform_id(): string;

  /** 判断本解析器是否能处理给定 URL。 */
  can_handle(url: string): boolean;

  /** 从 URL 中提取平台商品 ID；无法提取时抛出/返回 ITEM_ID_EXTRACT_FAILED。 */
  extract_item_id(url: string): string;

  /** 执行页面解析，返回结构化商品数据；Rust trait 与 ARCHITECTURE 4.3 对齐为 parse(&self, page: &Page) -> Result<ProductData>。 */
  parse(page: PageHandle): Promise<ProductData>;
}

/** 页面句柄抽象；Rust 实现直接使用 chromiumoxide::Page，本接口用于描述可测试能力边界。 */
export interface PageHandle {
  /** 获取当前页面 URL。 */
  url(): Promise<string>;

  /** 获取页面标题。 */
  title(): Promise<string>;

  /** 执行 JavaScript 并返回 JSON 值；Rust 对应 serde_json::Value。 */
  evaluate(script: string): Promise<JsonValue>;

  /** 获取当前 DOM HTML 快照；用于 raw.json 调试。 */
  content(): Promise<string>;
}

/** 可序列化页面快照；用于测试 mock、raw.json 和重解析，不替代 Rust chromiumoxide Page。 */
export interface PageContext {
  /** 当前页面 URL。 */
  url: string;

  /** 平台商品 ID（已由 extract_item_id 提取）。 */
  item_id: string;

  /** 页面标题。 */
  page_title: string;

  /**
   * CDP 执行 JavaScript 后的原始返回数据；Rust 对应 serde_json::Value。
   */
  raw_evaluate_result: JsonValue;

  /** 页面原始 HTML（可选保留，用于调试和重解析）。 */
  raw_html?: string;
}

/** 解析器初始化参数。 */
export interface ParserConfig {
  /** 是否保留原始 HTML 到 raw.json。 */
  keep_raw_html: boolean;

  /** 图片 URL 清洗策略；MVP 内置淘宝/京东规则，不可关闭。 */
  image_url_cleaning: boolean;
}

/** 解析结果包装器，用于内部传递原始数据与错误。 */
export interface ParseResult {
  /** 解析出的商品数据；完全失败时为 null。 */
  product: ProductData | null;

  /** 原始抓取数据，写入 raw.json 用于调试和重解析。 */
  raw_data: JsonObject;

  /** 解析过程中产生的错误和警告。 */
  errors: ScrapeErrorInfo[];
}
```

## 平台解析器特殊要求

### 淘宝 / 天猫解析器 (`taobao` / `tmall`)

| 数据项 | 提取来源 | 特殊规则 |
|--------|---------|---------|
| `title` | `g_config.idata.item.title` 或 DOM `h1` | 主标题 + 副标题拼接 |
| `cover` | `g_config.idata.item.images[0]` | 第一张主图作为封面 |
| `gallery` | `g_config.idata.item.images` | 所有主图 URL 列表 |
| `description.text` | `g_config.idata.item.desc` 或详情页文本提取 | 需去除 HTML 标签 |
| `description.html` | 原始详情 HTML（可选） | 保留用于调试 |
| `description.specs` | 参数表格 DOM 提取 | 键值对列表 |
| `detail_images` | 详情页图片 lazy-load 数据属性提取 | 需触发滚动或执行 JS 获取完整列表 |
| `skus` | `Hub.config.sku` 或 `g_config.idata.sku` | 规格名、规格值、价格、库存 |
| `sku_images` | SKU 缩略图映射 | key 为规格值 |
| `price` | `g_config.idata.item.price` / `sku.price` 区间计算 | 取所有 SKU 价格的 min/max |
| `shop` | `g_config.idata.seller` | 店铺名 + 店铺链接 |

**图片 URL 清洗规则（淘宝）**：
- 去除后缀 `_xxx.jpg` 中的尺寸标记，例如：
  - `https://img.example.com/abc_400x400.jpg` → `https://img.example.com/abc.jpg`
  - `https://img.example.com/abc_800x800.jpg` → `https://img.example.com/abc.jpg`
- 去除 `_q90` 等质量参数。
- 若清洗后 404，降级为原始 URL 并记录错误。

### 京东解析器 (`jd`)

| 数据项 | 提取来源 | 特殊规则 |
|--------|---------|---------|
| `title` | DOM `.sku-name` 或 `pageConfig.product.name` | 去除多余空白 |
| `cover` | `pageConfig.product.imageList[0]` | 第一张主图 |
| `gallery` | `pageConfig.product.imageList` | 所有主图 |
| `description.text` | 商品介绍区文本 | - |
| `description.html` | 原始详情 HTML（可选） | - |
| `description.specs` | 参数规格表 DOM | - |
| `detail_images` | 详情页图片 data-lazyload 或 data-src | 需滚动触发加载 |
| `skus` | `pageConfig.product.colorSize` / `pageConfig.product.skus` | 颜色/尺码等规格 |
| `sku_images` | 颜色缩略图映射 | key 为颜色名称 |
| `price` | `pageConfig.product.price` 或 AJAX 价格接口 | 取 SKU 价格区间 |
| `shop` | `pageConfig.shop` 或 DOM 提取 | 店铺名 + 链接 |

**图片 URL 清洗规则（京东）**：
- 去除 `s800x800_jfs/`、`s450x450_jfs/` 等尺寸前缀，例如：
  - `https://img10.360buyimg.com/n1/s800x800_jfs/t1/...jpg` → `https://img10.360buyimg.com/n1/jfs/t1/...jpg`
- 若清洗后 404，降级为原始 URL 并记录错误。

## 约束

- 所有解析器必须实现 `PlatformParser` 接口，字段名和类型不可擅自修改。
- `parse()` 方法是平台解析器 trait 的权威入口，成功时直接返回 `ProductData`；`ParseResult` 是 scraper/storage 内部包装器，用于携带 raw_data 与错误列表，不改变 ARCHITECTURE 4.3 的 trait 签名。
- `parse()` 方法返回的 `ProductData` 必须包含九个顶层字段：`title`, `cover`, `gallery`, `description`, `detail_images`, `skus`, `sku_images`, `price`, `shop`。
- 图片 URL 清洗失败时，必须保留原始 URL 到 `original_url`，记录错误到 `ParseResult.errors`，不可中断整体抓取流程。
- 解析失败时，`ParseResult.raw_data` 必须完整保留，供后续调试和重解析使用；Rust 类型为 `HashMap<String, serde_json::Value>` 或等价结构。
- 解析器不得依赖特定浏览器版本或用户代理字符串；仅通过 CDP 执行标准 JavaScript 和 DOM 查询。
- `can_handle(url)` 必须能准确识别平台域名：
  - 淘宝：`item.taobao.com`、`detail.tmall.com`
  - 京东：`item.jd.com`
- `extract_item_id(url)` 规则：淘宝/天猫从查询参数 `id` 提取非空数字串；京东从路径 `/{item_id}.html` 提取非空数字串；失败时返回 `ITEM_ID_EXTRACT_FAILED`，不得返回空字符串。
- URL 清洗规则：移除 fragment；保留识别 `item_id` 必需的 query；域名统一小写；协议优先规范化为 `https`；不得接受非目标域名的跳转 URL。

## 示例

```json
{
  "platform_id": "taobao",
  "item_id": "12345678",
  "parse_result": {
    "product": {
      "title": "示例连衣裙 夏季新款",
      "cover": {
        "original_url": "https://img.example.com/item/cover.jpg",
        "thumbnail_url": "https://img.example.com/item/cover_400x400.jpg",
        "local_path": null
      },
      "gallery": [...],
      "description": {
        "text": "夏季新款连衣裙，轻薄透气",
        "html": "<div>夏季新款连衣裙...</div>",
        "specs": [{ "key": "材质", "value": "棉" }]
      },
      "detail_images": [...],
      "skus": [
        {
          "name": "颜色",
          "value": "红色",
          "price": 99.0,
          "stock": 100,
          "image": {
            "original_url": "https://img.example.com/sku-red.jpg",
            "thumbnail_url": "https://img.example.com/sku-red_100x100.jpg",
            "local_path": null
          }
        }
      ],
      "sku_images": {
        "红色": {
          "original_url": "https://img.example.com/sku-red.jpg",
          "thumbnail_url": "https://img.example.com/sku-red_100x100.jpg",
          "local_path": null
        }
      },
      "price": { "min_price": 99.0, "max_price": 129.0, "currency": "CNY" },
      "shop": { "name": "示例店铺", "url": "https://shop.example.com" }
    },
    "raw_data": {
      "g_config": { "idata": { "item": { "title": "..." } } },
      "html_snapshot": "..."
    },
    "errors": [
      {
        "step": "parsing",
        "code": "DETAIL_IMAGE_PARTIAL",
        "message": "3 张详情图懒加载未触发",
        "recoverable": true
      }
    ]
  }
}
```
