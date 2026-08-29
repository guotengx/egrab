# EGrab 抓取规则包

**这个目录里的文件可以直接编辑，保存后下一次抓取立即生效，不需要重新编译、不需要重装程序。**

平台改版导致抓不到数据时，改这里就行。

---

## 文件说明

| 文件 | 作用 |
|------|------|
| `rules.json` | 规则总表：平台匹配、商品 ID 提取、等待条件、滚动参数、引用哪个 JS 文件 |
| `taobao.extract.js` | 淘宝/天猫 数据提取脚本 |
| `taobao.expand.js` | 淘宝/天猫 详情区展开脚本（滚动前执行） |
| `jd.extract.js` | 京东 数据提取脚本 |
| `jd.expand.js` | 京东 详情区展开脚本（滚动前执行） |
| `snapshots/` | 页面快照输出目录（用于排查平台改版） |

---

## 出问题时的标准排查流程

1. 在 EGrab 里打开【设置】→【导出页面快照】，程序会把当前浏览器页面的完整 HTML
   和所有候选全局变量存到 `snapshots/` 目录。
2. 打开对应任务文件夹里的 `raw.json`，看 `raw_data.debug` 段：
   - `galleryCount = 0` → 主图选择器失效
   - `specCount = 0` → 规格参数取值路径失效
   - `priceSamples = []` → 价格取值路径失效
   - `detailCount = 0` → 详情图选择器失效
3. 对照快照里的真实 DOM / JSON 结构，改对应的 `*.extract.js`。
4. 保存，回到 EGrab 重新抓取。**不需要重启程序。**

---

## `extract_js` 的返回值约定

脚本必须返回一个对象，字段与 `meta.json` 一一对应：

```js
{
  title: '商品标题',
  cover: 'https://...',              // 封面图；留空则自动取 gallery[0]
  gallery: ['https://...'],          // 主图集
  detail_images: ['https://...'],    // 详情图
  skus: [{ name: '颜色', value: '红色', price: 99.0, stock: 10, image: 'https://...' }],
  sku_images: { '红色': 'https://...' },
  price: { min: 99.0, max: 199.0, currency: 'CNY' },
  shop: { name: '店铺名', url: 'https://...' },
  description: { text: '纯文本', html: null, specs: [{ key: '品牌', value: 'xxx' }] },
  debug: { /* 任意诊断字段，会原样写进 raw.json */ }
}
```

图片 URL 只需返回页面上的原始地址，**去尺寸参数取原图由程序统一处理**
（`image_cleaner` 字段控制用哪套清洗规则：`taobao` / `jd` / `none`）。

---

## 抗改版写法建议

平台前端普遍使用构建哈希类名（例如京东的 `_gallery_116km_1`、`_scoped_1nhp8_1`），
**每次发版哈希都会变**。所以：

```js
// ❌ 会随平台发版失效
document.querySelectorAll('._gallery_116km_1 .image-carousel-track img')

// ✅ 子串匹配，哈希变了也能命中
document.querySelectorAll('[class*="gallery"] img, [class*="carousel"] img')
```

同理，读 SSR JSON 时不要写死路由名，做一次遍历兜底：

```js
var res = null;
try { res = ice.loaderData.home.data.res; } catch (e) {}
if (!res) { for (var k in ice.loaderData) { /* 找到含 item 的那个 */ } }
```

---

## 版本升级规则

`rules.json` 里的 `version` 是整数。程序启动时：

- 如果**内置规则版本 > 本地文件版本**，会把本地文件备份成 `*.bak`，然后覆盖为新版内置规则。
- 如果你手改了规则并希望**永久保留、不被升级覆盖**，把 `version` 改成一个很大的数（例如 `9999`）。

---

## 新增一个平台

在 `rules.json` 的 `platforms` 数组里加一项，再新建对应的 `xxx.extract.js` 即可，
不需要改任何 Rust 代码：

```jsonc
{
  "id": "1688",
  "label": "1688",
  "match_url": ["detail.1688.com"],
  "item_id": [{ "kind": "query", "key": "offerId" }, { "kind": "digit_run", "min_len": 8 }],
  "base_domain": "detail.1688.com",
  "image_cleaner": "taobao",
  "wait_js": "(function(){ return !!document.querySelector('[class*=\"title\"]'); })()",
  "extract_js_file": "1688.extract.js",
  "scroll": { "step": 600, "delay_ms": 300, "settle_ms": 1500, "max_height": 40000 }
}
```
