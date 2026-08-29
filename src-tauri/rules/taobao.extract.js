/*
 * EGrab 规则脚本 —— 淘宝 / 天猫 商品页数据提取
 *
 * 本文件在浏览器页面上下文中执行，返回值必须是一个对象，字段含义与 PRD 3.1.2 对齐：
 *   title           string                商品标题
 *   cover           string                封面图 URL
 *   gallery         string[]              主图集 URL 列表
 *   detail_images   string[]              详情图 URL 列表
 *   skus            {name,value,price,stock,image}[]
 *   sku_images      { [规格值]: URL }
 *   price           { min, max, currency }
 *   shop            { name, url }
 *   description     { text, html, specs: {key,value}[] }
 *   debug           object                任意诊断信息，会写进 raw.json
 *
 * 图片 URL 的"去尺寸参数取原图"由 Rust 侧统一处理，这里返回页面原始 URL 即可。
 *
 * ⚠️ 修改本文件后保存即生效，无需重新编译或重装程序。
 */
(function () {
  var out = {
    title: '',
    cover: '',
    gallery: [],
    detail_images: [],
    skus: [],
    sku_images: {},
    price: { min: 0, max: 0, currency: 'CNY' },
    shop: { name: '', url: '' },
    description: { text: '', html: null, specs: [] },
    debug: {}
  };

  function toAbs(u) {
    if (!u) return '';
    u = String(u).trim();
    if (!u) return '';
    if (u.indexOf('//') === 0) return 'https:' + u;
    return u;
  }
  function toNum(v) {
    if (v === null || v === undefined) return 0;
    var n = parseFloat(String(v).replace(/[^0-9.]/g, ''));
    return isNaN(n) ? 0 : n;
  }
  function pushUrl(arr, seen, u) {
    u = toAbs(u);
    if (!u || u.indexOf('data:') === 0) return;
    var key = u.replace(/^https?:/, '').split('?')[0];
    if (seen[key]) return;
    seen[key] = true;
    arr.push(u);
  }
  function qsa(sel) {
    try { return Array.prototype.slice.call(document.querySelectorAll(sel)); }
    catch (e) { return []; }
  }

  // ───────────────────────────────────────────────────────────
  // 1. 定位 ICE SSR 数据根节点
  //    正常路径：__ICE_APP_CONTEXT__.loaderData.home.data.res
  //    路由名可能不叫 home，因此增加遍历兜底。
  // ───────────────────────────────────────────────────────────
  var ice = window.__ICE_APP_CONTEXT__ || {};
  var res = null;
  try { res = ice.loaderData.home.data.res; } catch (e) {}
  if (!res && ice.loaderData) {
    for (var routeKey in ice.loaderData) {
      try {
        var d = ice.loaderData[routeKey].data;
        if (d && d.res && (d.res.item || d.res.componentsVO)) { res = d.res; break; }
      } catch (e) {}
    }
  }
  res = res || {};
  var item = res.item || {};
  var comp = res.componentsVO || {};

  out.debug.hasIce = !!window.__ICE_APP_CONTEXT__;
  out.debug.hasItem = !!res.item;
  out.debug.resKeys = Object.keys(res).slice(0, 40);

  // ───────────────────────────────────────────────────────────
  // 2. 标题
  // ───────────────────────────────────────────────────────────
  out.title = item.title || '';
  if (!out.title) { try { out.title = comp.titleVO.title.title || ''; } catch (e) {} }
  if (!out.title) {
    var tEl = document.querySelector('[class*="mainTitle"], [class*="ItemTitle"], [class*="itemTitle"], .tb-main-title');
    if (tEl) out.title = (tEl.textContent || '').trim();
  }
  if (!out.title) {
    out.title = (document.title || '').replace(/[-—]\s*(天猫|淘宝|Taobao|Tmall).*$/i, '').trim();
  }

  // ───────────────────────────────────────────────────────────
  // 3. 主图集 / 封面
  // ───────────────────────────────────────────────────────────
  var seenG = {};
  var picList = item.images || [];
  if (!picList || !picList.length) {
    try { picList = comp.headImageVO.images || []; } catch (e) { picList = []; }
  }
  for (var i = 0; i < picList.length; i++) pushUrl(out.gallery, seenG, picList[i]);

  if (!out.gallery.length) {
    // DOM 兜底：缩略图条
    qsa('[class*="thumbnail"] img, [class*="Thumbnail"] img, [class*="mainPic"] img, #J_UlThumb img').forEach(function (im) {
      pushUrl(out.gallery, seenG, im.getAttribute('src') || im.getAttribute('data-src'));
    });
  }
  out.cover = out.gallery.length ? out.gallery[0] : '';
  out.debug.galleryCount = out.gallery.length;

  // ───────────────────────────────────────────────────────────
  // 4. 价格
  //    真实结构（2026-08 实测）：
  //      res.skuCore.sku2info[skuId].price.priceMoney     = 19800  → 198.00 元
  //      res.skuCore.sku2info[skuId].subPrice.priceMoney  = 16806  → 168.06 元
  //    priceMoney 单位是"分"，优先使用它以避免文案字符串带干扰字符。
  // ───────────────────────────────────────────────────────────
  var prices = [];
  function collectPrice(p) {
    if (!p) return;
    if (p.priceMoney !== undefined && p.priceMoney !== null && String(p.priceMoney) !== '') {
      prices.push(toNum(p.priceMoney) / 100);
    } else if (p.priceText) {
      prices.push(toNum(p.priceText));
    }
  }
  try {
    var s2i = res.skuCore.sku2info || {};
    for (var skuKey in s2i) {
      collectPrice(s2i[skuKey].price);
      collectPrice(s2i[skuKey].subPrice);
      collectPrice(s2i[skuKey].promotionPrice);
    }
  } catch (e) {}
  if (!prices.length) {
    try { collectPrice(comp.priceVO.price); } catch (e) {}
    try { collectPrice(comp.priceVO.extraPrice); } catch (e) {}
  }
  if (!prices.length) {
    var pEl = document.querySelector('[class*="highlightPrice"] [class*="text"], [class*="Price--priceText"], [class*="priceText"], .tm-price, .tb-rmb-num');
    if (pEl) { var pv = toNum(pEl.textContent); if (pv > 0) prices.push(pv); }
  }
  prices = prices.filter(function (v) { return v > 0; });
  if (prices.length) {
    out.price.min = Math.min.apply(null, prices);
    out.price.max = Math.max.apply(null, prices);
  }
  out.debug.priceSamples = prices.slice(0, 8);

  // ───────────────────────────────────────────────────────────
  // 5. 店铺
  // ───────────────────────────────────────────────────────────
  var seller = res.seller || {};
  out.shop.name = seller.shopName || seller.sellerNick || '';
  out.shop.url = toAbs(seller.pcShopUrl || seller.shopUrl || '');
  if (!out.shop.name) {
    try {
      out.shop.name = comp.storeCardVO.shopName || '';
      out.shop.url = toAbs(comp.storeCardVO.shopUrl || '');
    } catch (e) {}
  }

  // ───────────────────────────────────────────────────────────
  // 6. 规格参数
  //    真实结构（2026-08 实测）：
  //      res.plusViewVO.industryParamVO.basicParamList[]  {propertyName, valueName}
  //      res.plusViewVO.industryParamVO.enhanceParamList[]
  //    兜底：componentsVO.extensionInfoVO.infos[type=BASE_PROPS].items[]
  // ───────────────────────────────────────────────────────────
  function addSpec(k, v) {
    k = String(k || '').trim();
    v = String(v || '').trim();
    if (k && v) out.description.specs.push({ key: k, value: v });
  }
  try {
    var ip = res.plusViewVO.industryParamVO || {};
    (ip.basicParamList || []).forEach(function (p) { addSpec(p.propertyName, p.valueName); });
    (ip.enhanceParamList || []).forEach(function (p) { addSpec(p.propertyName, p.valueName); });
  } catch (e) {}
  if (!out.description.specs.length) {
    try {
      (comp.extensionInfoVO.infos || []).forEach(function (info) {
        if (info.type === 'BASE_PROPS') {
          (info.items || []).forEach(function (it) {
            addSpec(it.title, (it.text || []).join(' '));
          });
        }
      });
    } catch (e) {}
  }
  if (!out.description.specs.length) {
    qsa('#J_AttrUL li, .attributes-list li, [class*="attr"] li').forEach(function (li) {
      var text = (li.textContent || '').trim();
      var idx = text.indexOf(':');
      if (idx < 0) idx = text.indexOf('：');
      if (idx > 0) addSpec(text.substring(0, idx), text.substring(idx + 1));
    });
  }
  out.debug.specCount = out.description.specs.length;

  // ───────────────────────────────────────────────────────────
  // 7. SKU
  //    真实结构（2026-08 实测）：
  //      res.skuBase.props[]  {pid, name, values:[{vid, name, image?}]}
  //      res.skuBase.skus[]   {propPath:"pid:vid;pid:vid", skuId}
  //      res.skuCore.sku2info[skuId] {price:{priceMoney}, quantity}
  // ───────────────────────────────────────────────────────────
  try {
    var skuBase = res.skuBase || {};
    var s2iMap = (res.skuCore && res.skuCore.sku2info) || {};
    var pathToSkuId = {};
    (skuBase.skus || []).forEach(function (s) {
      if (s && s.propPath) pathToSkuId[s.propPath] = s.skuId;
    });
    (skuBase.props || []).forEach(function (prop) {
      (prop.values || []).forEach(function (v) {
        var pvPath = String(prop.pid) + ':' + String(v.vid);
        // 单属性商品：propPath 就是 pid:vid；多属性时做前缀匹配
        var skuId = pathToSkuId[pvPath];
        if (!skuId) {
          for (var pp in pathToSkuId) {
            if (pp.indexOf(pvPath) >= 0) { skuId = pathToSkuId[pp]; break; }
          }
        }
        var info = skuId ? s2iMap[skuId] : null;
        var price = 0;
        var stock = null;
        if (info) {
          if (info.price && info.price.priceMoney) price = toNum(info.price.priceMoney) / 100;
          else if (info.price && info.price.priceText) price = toNum(info.price.priceText);
          if (typeof info.quantity === 'number') stock = info.quantity;
        }
        var imgUrl = toAbs(v.image || '');
        out.skus.push({
          name: prop.name || '',
          value: v.name || '',
          price: price,
          stock: stock,
          image: imgUrl
        });
        if (imgUrl && v.name) out.sku_images[v.name] = imgUrl;
      });
    });
  } catch (e) {
    out.debug.skuError = String(e);
  }
  out.debug.skuCount = out.skus.length;

  // ───────────────────────────────────────────────────────────
  // 8. 详情图（图文详情区）
  // ───────────────────────────────────────────────────────────
  var seenD = {};
  var descSel = [
    '#imageTextInfo-container img',
    '[class*="descV8"] img',
    '[class*="desc-root"] img',
    '[class*="descriptionn"] img',
    '#description img',
    '.desc-detail img',
    '[id*="detail"] img',
    '[class*="detailDesc"] img'
  ].join(',');
  qsa(descSel).forEach(function (im) {
    var src = im.getAttribute('data-src') || im.getAttribute('src') || im.getAttribute('data-lazyload-src');
    if (!src) return;
    if (!/alicdn\.com|taobao\.com|tmall\.com/i.test(src)) return;
    if (!/\.(jpg|jpeg|png|webp|avif|gif|bmp)($|\?|#|_|!)/i.test(src)) return;
    if (/(icon|sprite|avatar|logo)/i.test(src)) return;
    pushUrl(out.detail_images, seenD, src);
  });
  out.debug.detailCount = out.detail_images.length;

  // ───────────────────────────────────────────────────────────
  // 9. 描述文本（用规格参数拼装，保证 meta.json 非空）
  // ───────────────────────────────────────────────────────────
  out.description.text = out.description.specs
    .map(function (s) { return s.key + ': ' + s.value; })
    .join('\n');

  return out;
})()
