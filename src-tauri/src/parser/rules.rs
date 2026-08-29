// EGrab - Parser Module: External Rule Pack Engine
//
// 把「抓什么、怎么抓」从 Rust 二进制里剥离出来，改为磁盘上的可编辑规则包。
// 平台改版时只需修改规则文件，无需重新编译或重装程序。
//
// 加载优先级：
//   1. 磁盘规则目录（用户可编辑）  <app_data>/rules/
//   2. 内嵌默认规则（编译进二进制，永远可用的兜底）
//
// 升级语义：
//   内嵌版本号 > 磁盘版本号 时，磁盘文件备份为 *.bak 后被覆盖。
//   用户若想固定自己的规则，把 rules.json 的 version 改成一个很大的数即可。

use crate::models::{
    Description, ImageRef, PriceRange, ProductData, ScrapeErrorInfo, ScrapeStep, ShopInfo, SkuItem,
    SpecItem,
};
use crate::parser::utils::{
    build_default_product, clean_jd_image_url, clean_taobao_image_url, normalize_image_url,
};
use crate::parser::{PageHandle, ParseResult, PlatformParser};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

// ---- 内嵌默认规则包 ----------------------------------------------------

const EMBEDDED_RULES_JSON: &str = include_str!("../../rules/rules.json");

/// 内嵌的规则文件表：(文件名, 内容)。
/// 新增平台脚本时在这里追加一行即可。
const EMBEDDED_FILES: &[(&str, &str)] = &[
    ("rules.json", EMBEDDED_RULES_JSON),
    ("README.md", include_str!("../../rules/README.md")),
    (
        "taobao.extract.js",
        include_str!("../../rules/taobao.extract.js"),
    ),
    (
        "taobao.expand.js",
        include_str!("../../rules/taobao.expand.js"),
    ),
    ("jd.extract.js", include_str!("../../rules/jd.extract.js")),
    ("jd.expand.js", include_str!("../../rules/jd.expand.js")),
];

// ---- 规则数据结构 ------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePack {
    pub version: u32,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub platforms: Vec<PlatformRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRule {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub match_url: Vec<String>,
    #[serde(default)]
    pub item_id: Vec<ItemIdRule>,
    #[serde(default)]
    pub base_domain: String,
    #[serde(default)]
    pub image_cleaner: String,
    #[serde(default)]
    pub wait_js: Option<String>,
    #[serde(default)]
    pub expand_js: Option<String>,
    #[serde(default)]
    pub expand_js_file: Option<String>,
    #[serde(default)]
    pub extract_js: Option<String>,
    #[serde(default)]
    pub extract_js_file: Option<String>,
    #[serde(default)]
    pub scroll: ScrollRule,
}

/// 商品 ID 提取规则。
/// kind:
///   "query"       — 从 query string 取 `key` 参数（必须是纯数字）
///   "path_digits" — 取路径最后一段 `.html` 之前的数字
///   "digit_run"   — 取路径中最长的一段数字，长度需 >= min_len
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemIdRule {
    pub kind: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub min_len: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollRule {
    #[serde(default = "default_scroll_step")]
    pub step: u32,
    #[serde(default = "default_scroll_delay")]
    pub delay_ms: u64,
    #[serde(default = "default_scroll_settle")]
    pub settle_ms: u64,
    #[serde(default = "default_scroll_max_height")]
    pub max_height: u32,
}

fn default_scroll_step() -> u32 {
    500
}
fn default_scroll_delay() -> u64 {
    300
}
fn default_scroll_settle() -> u64 {
    1500
}
fn default_scroll_max_height() -> u32 {
    60000
}

impl Default for ScrollRule {
    fn default() -> Self {
        Self {
            step: default_scroll_step(),
            delay_ms: default_scroll_delay(),
            settle_ms: default_scroll_settle(),
            max_height: default_scroll_max_height(),
        }
    }
}

// ---- 路径解析 ----------------------------------------------------------

/// 应用数据目录（与 index.db 同级）。
///   macOS:   ~/Library/Application Support/com.egrab.app
///   Windows: %APPDATA%\com.egrab.app
pub fn app_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.egrab.app")
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string());
        PathBuf::from(appdata).join("com.egrab.app")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".egrab")
    }
}

/// 规则包目录。
pub fn rules_dir() -> PathBuf {
    app_data_dir().join("rules")
}

/// 页面快照输出目录。
pub fn snapshots_dir() -> PathBuf {
    rules_dir().join("snapshots")
}

// ---- 规则包加载 --------------------------------------------------------

/// 解析内嵌规则包。内嵌 JSON 是随二进制发布的，格式错误属于构建期问题。
fn embedded_pack() -> RulePack {
    // 内嵌 JSON 由本仓库维护并随二进制编译，解析失败说明规则文件写坏了，
    // 属于必须在 CI 阶段暴露的构建错误，因此这里使用 expect 是可接受的。
    serde_json::from_str::<RulePack>(EMBEDDED_RULES_JSON)
        .expect("embedded rules.json is malformed (build-time error)")
}

/// 首次运行时把内嵌规则释放到磁盘；内嵌版本更高时升级并备份旧文件。
///
/// 返回一句人类可读的说明，用于日志。
pub fn ensure_rules_on_disk() -> String {
    let dir = rules_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return format!("failed to create rules dir {}: {}", dir.display(), e);
    }
    let _ = std::fs::create_dir_all(snapshots_dir());

    let embedded_version = embedded_pack().version;
    let disk_rules_path = dir.join("rules.json");

    let disk_version = std::fs::read_to_string(&disk_rules_path)
        .ok()
        .and_then(|t| serde_json::from_str::<RulePack>(&t).ok())
        .map(|p| p.version);

    let should_overwrite = match disk_version {
        None => true,
        Some(v) => embedded_version > v,
    };

    for (name, content) in EMBEDDED_FILES {
        let path = dir.join(name);
        let exists = path.exists();
        if exists && !should_overwrite {
            continue;
        }
        if exists {
            // 覆盖前备份，绝不静默丢失用户改动。
            let backup = dir.join(format!("{}.bak", name));
            let _ = std::fs::copy(&path, &backup);
        }
        if let Err(e) = std::fs::write(&path, content) {
            tracing::warn!(file = %path.display(), error = %e, "Failed to write rule file");
        }
    }

    match disk_version {
        None => format!(
            "rules initialized at {} (v{})",
            dir.display(),
            embedded_version
        ),
        Some(v) if should_overwrite => format!(
            "rules upgraded {} -> v{} (old files backed up as *.bak)",
            v, embedded_version
        ),
        Some(v) => format!("rules kept from disk (v{})", v),
    }
}

/// 规则包来源，用于诊断展示。
#[derive(Debug, Clone, Serialize)]
pub struct RuleSource {
    pub source: String,
    pub path: String,
    pub version: u32,
    pub platforms: Vec<String>,
    pub error: Option<String>,
}

/// 读取当前生效的规则包。
///
/// 磁盘文件优先；磁盘文件缺失或 JSON 非法时自动回退到内嵌规则，
/// 保证「改坏规则文件不会让程序变砖」。
pub fn load_rule_pack() -> (RulePack, RuleSource) {
    let dir = rules_dir();
    let path = dir.join("rules.json");

    let mut error: Option<String> = None;

    if let Ok(text) = std::fs::read_to_string(&path) {
        match serde_json::from_str::<RulePack>(&text) {
            Ok(mut pack) => {
                resolve_script_files(&mut pack, &dir);
                let src = RuleSource {
                    source: "disk".to_string(),
                    path: path.display().to_string(),
                    version: pack.version,
                    platforms: pack.platforms.iter().map(|p| p.id.clone()).collect(),
                    error: None,
                };
                return (pack, src);
            }
            Err(e) => {
                let msg = format!("rules.json parse error: {}", e);
                tracing::warn!("{} — falling back to embedded rules", msg);
                error = Some(msg);
            }
        }
    }

    let mut pack = embedded_pack();
    resolve_script_files(&mut pack, &dir);
    let src = RuleSource {
        source: "embedded".to_string(),
        path: path.display().to_string(),
        version: pack.version,
        platforms: pack.platforms.iter().map(|p| p.id.clone()).collect(),
        error,
    };
    (pack, src)
}

/// 把 `*_js_file` 引用的脚本读进内存：磁盘优先，其次内嵌表。
fn resolve_script_files(pack: &mut RulePack, dir: &PathBuf) {
    for platform in pack.platforms.iter_mut() {
        if platform.extract_js.is_none() {
            if let Some(ref file) = platform.extract_js_file {
                platform.extract_js = read_script(dir, file);
            }
        }
        if platform.expand_js.is_none() {
            if let Some(ref file) = platform.expand_js_file {
                platform.expand_js = read_script(dir, file);
            }
        }
    }
}

fn read_script(dir: &PathBuf, file: &str) -> Option<String> {
    // 防目录穿越：只接受纯文件名。
    if file.contains('/') || file.contains('\\') || file.contains("..") {
        tracing::warn!(file = file, "Rejected script file name (path traversal)");
        return None;
    }
    if let Ok(text) = std::fs::read_to_string(dir.join(file)) {
        return Some(text);
    }
    EMBEDDED_FILES
        .iter()
        .find(|(name, _)| *name == file)
        .map(|(_, content)| content.to_string())
}

/// 查找能处理该 URL 的平台规则。
pub fn find_rule(url: &str) -> Option<PlatformRule> {
    let (pack, _) = load_rule_pack();
    let lower = url.to_lowercase();
    pack.platforms
        .into_iter()
        .find(|p| p.match_url.iter().any(|m| lower.contains(&m.to_lowercase())))
}

// ---- 规则驱动的解析器 --------------------------------------------------

/// 由外置规则驱动的通用平台解析器。
/// 所有平台差异都在规则文件里，本结构体不含任何平台专属逻辑。
pub struct RuleParser {
    rule: PlatformRule,
}

impl RuleParser {
    pub fn new(rule: PlatformRule) -> Self {
        Self { rule }
    }

    pub fn rule(&self) -> &PlatformRule {
        &self.rule
    }

    /// 按平台规则清洗图片 URL（去尺寸参数取原图）并补全为绝对地址。
    fn clean_url(&self, url: &str) -> String {
        let cleaned = match self.rule.image_cleaner.as_str() {
            "taobao" => clean_taobao_image_url(url),
            "jd" => clean_jd_image_url(url),
            _ => url.to_string(),
        };
        let base = if self.rule.base_domain.is_empty() {
            "localhost"
        } else {
            self.rule.base_domain.as_str()
        };
        normalize_image_url(&cleaned, base)
    }

    fn to_image_ref(&self, url: &str) -> ImageRef {
        ImageRef {
            original_url: self.clean_url(url),
            thumbnail_url: url.to_string(),
            local_path: None,
        }
    }
}

/// 从 JSON 值里取字符串形式的 URL：支持裸字符串，也支持 `{ "url": "..." }`。
fn value_to_url(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        let t = s.trim();
        if t.is_empty() {
            return None;
        }
        return Some(t.to_string());
    }
    v.get("url")
        .and_then(|u| u.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 宽容地把 JSON 值转成 f64：接受数字，也接受 "168.06" / "￥168.06" 这类字符串。
fn value_to_f64(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => {
            let filtered: String = s
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            filtered.parse::<f64>().unwrap_or(0.0)
        }
        _ => 0.0,
    }
}

fn value_to_string(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

#[async_trait]
impl PlatformParser for RuleParser {
    fn platform_id(&self) -> &str {
        &self.rule.id
    }

    fn can_handle(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        self.rule
            .match_url
            .iter()
            .any(|m| lower.contains(&m.to_lowercase()))
    }

    fn extract_item_id(&self, url: &str) -> anyhow::Result<String> {
        let without_hash = url.split('#').next().unwrap_or(url);
        let mut parts = without_hash.splitn(2, '?');
        let path = parts.next().unwrap_or("");
        let query = parts.next().unwrap_or("");

        for rule in &self.rule.item_id {
            match rule.kind.as_str() {
                "query" => {
                    for pair in query.split('&') {
                        let mut kv = pair.splitn(2, '=');
                        let k = kv.next().unwrap_or("");
                        let v = kv.next().unwrap_or("");
                        if k == rule.key
                            && !v.is_empty()
                            && v.chars().all(|c| c.is_ascii_digit())
                        {
                            return Ok(v.to_string());
                        }
                    }
                }
                "path_digits" => {
                    let last = path.rsplit('/').next().unwrap_or("");
                    if let Some(pos) = last.find(".html") {
                        let id = &last[..pos];
                        if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                            return Ok(id.to_string());
                        }
                    }
                }
                "digit_run" => {
                    let min_len = if rule.min_len == 0 { 5 } else { rule.min_len };
                    let mut best = String::new();
                    let mut current = String::new();
                    for ch in path.chars() {
                        if ch.is_ascii_digit() {
                            current.push(ch);
                        } else {
                            if current.len() > best.len() {
                                best = current.clone();
                            }
                            current.clear();
                        }
                    }
                    if current.len() > best.len() {
                        best = current;
                    }
                    if best.len() >= min_len {
                        return Ok(best);
                    }
                }
                other => {
                    tracing::warn!(kind = other, "Unknown item_id rule kind; ignored");
                }
            }
        }

        anyhow::bail!(
            "Failed to extract item ID from URL using platform '{}' rules: {}",
            self.rule.id,
            url
        )
    }

    async fn parse(&self, page: &dyn PageHandle) -> anyhow::Result<ParseResult> {
        let mut errors: Vec<ScrapeErrorInfo> = Vec::new();

        let script = match self.rule.extract_js {
            Some(ref s) if !s.trim().is_empty() => s.clone(),
            _ => {
                anyhow::bail!(
                    "Platform '{}' has no extract_js defined in the rule pack",
                    self.rule.id
                );
            }
        };

        let raw = page.evaluate(&script).await.unwrap_or(Value::Null);

        if raw.is_null() {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "EXTRACT_JS_RETURNED_NULL".to_string(),
                message: format!(
                    "Rule script for '{}' returned null; the page structure may have changed",
                    self.rule.id
                ),
                recoverable: true,
            });
        }

        // ---- 标题 ----
        let mut title = value_to_string(raw.get("title"));
        if title.is_empty() {
            if let Ok(page_title) = page.title().await {
                title = page_title.trim().to_string();
            }
        }
        if title.is_empty() {
            title = "Unknown Title".to_string();
        }

        let mut product: ProductData = build_default_product(title);

        // ---- 主图集 ----
        if let Some(arr) = raw.get("gallery").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(url) = value_to_url(item) {
                    product.gallery.push(self.to_image_ref(&url));
                }
            }
        }

        // ---- 封面：显式指定优先，否则取主图第一张 ----
        let cover_url = raw.get("cover").and_then(value_to_url);
        match cover_url {
            Some(url) => product.cover = self.to_image_ref(&url),
            None => {
                // 先 clone 出来结束对 product.gallery 的借用，再写入 product.cover。
                let first = product.gallery.first().cloned();
                if let Some(img) = first {
                    product.cover = img;
                }
            }
        }

        // ---- 详情图 ----
        if let Some(arr) = raw.get("detail_images").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(url) = value_to_url(item) {
                    product.detail_images.push(self.to_image_ref(&url));
                }
            }
        }

        // ---- SKU ----
        if let Some(arr) = raw.get("skus").and_then(|v| v.as_array()) {
            for item in arr {
                let image = item.get("image").and_then(value_to_url).map(|u| self.to_image_ref(&u));
                let stock = item
                    .get("stock")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
                product.skus.push(SkuItem {
                    name: value_to_string(item.get("name")),
                    value: value_to_string(item.get("value")),
                    price: value_to_f64(item.get("price")),
                    stock,
                    image,
                });
            }
        }

        // ---- SKU 图 ----
        if let Some(obj) = raw.get("sku_images").and_then(|v| v.as_object()) {
            for (name, val) in obj {
                if let Some(url) = value_to_url(val) {
                    product.sku_images.insert(name.clone(), self.to_image_ref(&url));
                }
            }
        }

        // ---- 价格 ----
        if let Some(price) = raw.get("price") {
            let min = value_to_f64(price.get("min").or_else(|| price.get("min_price")));
            let max = value_to_f64(price.get("max").or_else(|| price.get("max_price")));
            let currency = {
                let c = value_to_string(price.get("currency"));
                if c.is_empty() {
                    "CNY".to_string()
                } else {
                    c
                }
            };
            let (min, max) = if max <= 0.0 && min > 0.0 {
                (min, min)
            } else if min <= 0.0 && max > 0.0 {
                (max, max)
            } else {
                (min, max)
            };
            product.price = PriceRange {
                min_price: min,
                max_price: max,
                currency,
            };
        }

        // ---- 店铺 ----
        if let Some(shop) = raw.get("shop") {
            product.shop = ShopInfo {
                name: value_to_string(shop.get("name")),
                url: value_to_string(shop.get("url")),
            };
        }

        // ---- 描述与规格参数 ----
        if let Some(desc) = raw.get("description") {
            let mut specs: Vec<SpecItem> = Vec::new();
            if let Some(arr) = desc.get("specs").and_then(|v| v.as_array()) {
                for item in arr {
                    let key = value_to_string(item.get("key"));
                    let value = value_to_string(item.get("value"));
                    if !key.is_empty() {
                        specs.push(SpecItem { key, value });
                    }
                }
            }
            product.description = Description {
                text: value_to_string(desc.get("text")),
                html: desc
                    .get("html")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                specs,
            };
        }

        // ---- 诊断信息落进 raw.json ----
        let mut raw_data = serde_json::Map::new();
        raw_data.insert("rule_platform".to_string(), Value::String(self.rule.id.clone()));
        raw_data.insert(
            "debug".to_string(),
            raw.get("debug").cloned().unwrap_or(Value::Null),
        );
        raw_data.insert(
            "counts".to_string(),
            serde_json::json!({
                "gallery": product.gallery.len(),
                "detail_images": product.detail_images.len(),
                "skus": product.skus.len(),
                "specs": product.description.specs.len(),
                "price_min": product.price.min_price,
                "price_max": product.price.max_price,
            }),
        );

        // ---- 降级告警：缺字段不中断流程，只记录 ----
        if product.gallery.is_empty() {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "GALLERY_EMPTY".to_string(),
                message: format!(
                    "未抓到主图。可能是平台改版，请编辑规则文件 {} 中的主图选择器",
                    self.rule
                        .extract_js_file
                        .clone()
                        .unwrap_or_else(|| "extract_js".to_string())
                ),
                recoverable: true,
            });
        }
        if product.detail_images.is_empty() {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "DETAIL_IMAGES_EMPTY".to_string(),
                message: "未抓到详情图；懒加载可能未触发，或详情容器选择器已失效".to_string(),
                recoverable: true,
            });
        }
        if product.price.max_price <= 0.0 {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "PRICE_MISSING".to_string(),
                message: "未抓到价格；请检查规则文件中的价格取值路径".to_string(),
                recoverable: true,
            });
        }
        if product.description.specs.is_empty() {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "SPECS_EMPTY".to_string(),
                message: "未抓到规格参数；请检查规则文件中的参数取值路径".to_string(),
                recoverable: true,
            });
        }

        Ok(ParseResult {
            product: Some(product),
            raw_data: Value::Object(raw_data),
            errors,
        })
    }
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> RulePack {
        embedded_pack()
    }

    fn parser_for(id: &str) -> RuleParser {
        let rule = pack()
            .platforms
            .into_iter()
            .find(|p| p.id == id)
            .expect("platform must exist in embedded rules");
        RuleParser::new(rule)
    }

    #[test]
    fn embedded_rules_are_valid_json() {
        let p = pack();
        assert!(p.version >= 1);
        assert!(p.platforms.len() >= 2, "expect taobao + jd");
    }

    #[test]
    fn embedded_rules_reference_existing_scripts() {
        let mut p = pack();
        // 传入一个不存在的目录，强制走内嵌文件表。
        resolve_script_files(&mut p, &PathBuf::from("/nonexistent-egrab-rules-dir"));
        for platform in &p.platforms {
            assert!(
                platform
                    .extract_js
                    .as_ref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false),
                "platform {} has no resolvable extract_js",
                platform.id
            );
        }
    }

    #[test]
    fn taobao_can_handle() {
        let p = parser_for("taobao");
        assert!(p.can_handle("https://item.taobao.com/item.htm?id=123456"));
        assert!(p.can_handle("https://detail.tmall.com/item.htm?id=789"));
        assert!(p.can_handle("https://chaoshi.detail.tmall.com/item.htm?id=111"));
        assert!(!p.can_handle("https://item.jd.com/12345678.html"));
    }

    #[test]
    fn jd_can_handle() {
        let p = parser_for("jd");
        assert!(p.can_handle("https://item.jd.com/12345678.html"));
        assert!(!p.can_handle("https://item.taobao.com/item.htm?id=1"));
        assert!(!p.can_handle("https://www.jd.com/"));
    }

    #[test]
    fn taobao_item_id() {
        let p = parser_for("taobao");
        assert_eq!(
            p.extract_item_id("https://item.taobao.com/item.htm?id=123456789")
                .unwrap(),
            "123456789"
        );
        assert_eq!(
            p.extract_item_id("https://detail.tmall.com/item.htm?id=987654321&spm=xxx")
                .unwrap(),
            "987654321"
        );
        assert!(p
            .extract_item_id("https://item.taobao.com/item.htm?no_id=123")
            .is_err());
        assert!(p
            .extract_item_id("https://item.taobao.com/item.htm?id=abc123")
            .is_err());
    }

    #[test]
    fn jd_item_id() {
        let p = parser_for("jd");
        assert_eq!(
            p.extract_item_id("https://item.jd.com/12345678.html").unwrap(),
            "12345678"
        );
        assert_eq!(
            p.extract_item_id("https://item.jd.com/12345678.html?spm=xxx")
                .unwrap(),
            "12345678"
        );
    }

    #[test]
    fn value_to_f64_accepts_strings_and_numbers() {
        assert_eq!(value_to_f64(Some(&serde_json::json!(198.0))), 198.0);
        assert_eq!(value_to_f64(Some(&serde_json::json!("￥168.06"))), 168.06);
        assert_eq!(value_to_f64(None), 0.0);
    }

    #[test]
    fn value_to_url_accepts_string_and_object() {
        assert_eq!(
            value_to_url(&serde_json::json!("https://a.com/1.jpg")),
            Some("https://a.com/1.jpg".to_string())
        );
        assert_eq!(
            value_to_url(&serde_json::json!({"url": "https://a.com/2.jpg"})),
            Some("https://a.com/2.jpg".to_string())
        );
        assert_eq!(value_to_url(&serde_json::json!("")), None);
    }

    #[test]
    fn read_script_rejects_path_traversal() {
        let dir = PathBuf::from("/nonexistent-egrab-rules-dir");
        assert!(read_script(&dir, "../secret.js").is_none());
        assert!(read_script(&dir, "sub/dir.js").is_none());
    }
}
