/*
 * EGrab 规则脚本 —— 京东 商品页数据提取
 *
 * 返回值字段说明见 taobao.extract.js 顶部注释。
 *
 * 【重要设计原则】
 * 京东前端使用 CSS Module 哈希类名（如 _gallery_116km_1、_scoped_1nhp8_1），
 * 哈希值每次发版都会变化。因此本文件一律使用 [class*="关键词"] 子串选择器，
 * 而不是写死完整类名，这样京东改哈希也不会失效。
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
    u = String(u).trim().replace(/^["']+|["']+$/g, '');
    if (!u) return '';
    if (/^https?:\/\//i.test(u)) return u;
    if (u.indexOf('//') === 0) return 'https:' + u;
    if (u.charAt(0) !== '/') u = '/' + u;
    return 'https://img30.360buyimg.com' + u;
  }
  function toNum(v) {
    if (v === null || v === undefined) return 0;
    var n = parseFloat(String(v).replace(/[^0-9.]/g, ''));
    return isNaN(n) ? 0 : n;
  }
  function qsa(sel) {
    try { return Array.prototype.slice.call(document.querySelectorAll(sel)); }
    catch (e) { return []; }
  }
  // 京东图片同一张图会有多种尺寸前缀（n1/s450x450_jfs/...、n5/s54x54_jfs/...），
  // 用 jfs/ 之后的路径做去重键，保证同一张图只保留一次。
  function jdKey(u) {
    var i = u.indexOf('jfs/');
    return (i >= 0 ? u.slice(i) : u).split('?')[0].split('!')[0];
  }
  function isJdImage(u) {
    if (!u || u.indexOf('data:') === 0) return false;
    if (!/360buyimg\.com|jd\.com/i.test(u)) return false;
    if (!/\.(jpg|jpeg|png|webp|avif|gif|bmp)($|\?|#|!)/i.test(u)) return false;
    if (/\/(icon|tool|sprite|avatar|logo)/i.test(u)) return false;
    return true;
  }
  function pushImg(arr, seen, raw) {
    var u = toAbs(raw);
    if (!isJdImage(u)) return;
    var k = jdKey(u);
    if (seen[k]) return;
    seen[k] = true;
    arr.push(u);
  }

  // ───────────────────────────────────────────────────────────
  // 1. 标题
  // ───────────────────────────────────────────────────────────
  var tEl = document.querySelector('.sku-name, [class*="sku-title-name"], [class*="itemName"], [class*="ItemName"], #name h1, #itemName');
  out.title = tEl ? (tEl.textContent || '').trim() : '';
  if (!out.title) {
    out.title = (document.title || '')
      .replace(/【[^】]*】/g, '')
      .replace(/[-—]\s*京东.*$/, '')
      .trim();
  }

  // ───────────────────────────────────────────────────────────
  // 2. 主图集 / 封面
  //    2026-08 起 pageConfig.product 已不再包含 imageList，只能走 DOM。
  //    使用 [class*=] 子串匹配抵抗构建哈希变化。
  // ───────────────────────────────────────────────────────────
  var seenG = {};
  var gallerySel = [
    '[class*="gallery"] img',
    '[class*="Gallery"] img',
    '[class*="carousel"] img',
    '[class*="Carousel"] img',
    '[class*="preview"] img',
    '[class*="mainImage"] img',
    '[class*="main-img"] img',
    '#spec-list img',
    '#spec-n1 img',
    '#preview img'
  ].join(',');
  qsa(gallerySel).forEach(function (im) {
    pushImg(out.gallery, seenG, im.getAttribute('src') || im.getAttribute('data-src') || im.getAttribute('data-lazy-img') || im.getAttribute('data-origin'));
  });
  out.debug.gallerySelectorHits = out.gallery.length;

  // 兜底：整页扫描 item_pic / n1 尺寸的主图（排除详情区）
  if (!out.gallery.length) {
    var detailRoot = document.querySelector('#detail-main, #detail, [class*="scoped"]');
    qsa('img').forEach(function (im) {
      if (out.gallery.length >= 15) return;
      if (detailRoot && detailRoot.contains(im)) return;
      var src = im.getAttribute('src') || im.getAttribute('data-src') || '';
      if (!/\/n\d\/|s\d+x\d+_jfs/i.test(src)) return;
      pushImg(out.gallery, seenG, src);
    });
    out.debug.galleryFallbackUsed = true;
  }
  if (out.gallery.length > 15) out.gallery = out.gallery.slice(0, 15);
  out.cover = out.gallery.length ? out.gallery[0] : '';
  out.debug.galleryCount = out.gallery.length;

  // ───────────────────────────────────────────────────────────
  // 3. 价格
  // ───────────────────────────────────────────────────────────
  var priceVal = 0;
  var priceSel = [
    '.product-price--value',
    '[class*="product-price"] [class*="value"]',
    '[class*="priceValue"]',
    '[class*="price-value"]',
    '#jd-price',
    '.p-price .price',
    '.summary-price .p-price'
  ].join(',');
  var pEl = document.querySelector(priceSel);
  if (pEl) priceVal = toNum(pEl.textContent);

  if (!priceVal) {
    // 通用兜底：找 class 含 price 且文本形如 ¥123.45 的元素
    var cands = qsa('[class*="price"], [class*="Price"]');
    for (var ci = 0; ci < cands.length && !priceVal; ci++) {
      var txt = (cands[ci].textContent || '').trim();
      if (txt.length > 15) continue;
      if (!/^[¥￥]?\s*\d+(\.\d{1,2})?$/.test(txt)) continue;
      var v = toNum(txt);
      if (v > 0) priceVal = v;
    }
  }
  if (priceVal > 0) { out.price.min = priceVal; out.price.max = priceVal; }
  out.debug.priceValue = priceVal;

  // ───────────────────────────────────────────────────────────
  // 4. 店铺
  // ───────────────────────────────────────────────────────────
  var shopEl = document.querySelector('[class*="shop-name"] a, [class*="shopName"] a, [class*="shop-name"], [class*="shopName"], .top-name, #popbox .mname a');
  if (shopEl) {
    out.shop.name = (shopEl.textContent || '').trim();
    var href = shopEl.getAttribute && shopEl.getAttribute('href');
    if (href) out.shop.url = href.indexOf('//') === 0 ? 'https:' + href : href;
  }

  // ───────────────────────────────────────────────────────────
  // 5. 规格参数
  // ───────────────────────────────────────────────────────────
  function addSpec(k, v) {
    k = String(k || '').trim().replace(/[:：]\s*$/, '');
    v = String(v || '').trim();
    if (k && v) out.description.specs.push({ key: k, value: v });
  }
  qsa('.attrs .item, [class*="attrs"] [class*="item"]').forEach(function (it) {
    var label = it.querySelector('.label .text') || it.querySelector('.label') || it.querySelector('[class*="label"]');
    var value = it.querySelector('.value .text') || it.querySelector('.value') || it.querySelector('[class*="value"]');
    if (label && value) addSpec(label.textContent, value.textContent);
  });
  if (!out.description.specs.length) {
    qsa('.parameter2 li, [class*="parameter"] li, #detail .p-parameter li').forEach(function (li) {
      var text = (li.textContent || '').trim();
      var idx = text.indexOf('：');
      if (idx < 0) idx = text.indexOf(':');
      if (idx > 0) addSpec(text.substring(0, idx), text.substring(idx + 1));
    });
  }
  out.debug.specCount = out.description.specs.length;

  // ───────────────────────────────────────────────────────────
  // 6. SKU 图（规格缩略图）
  // ───────────────────────────────────────────────────────────
  qsa('.specification-item-sku-image, [class*="sku-image"] img, [class*="specification"] img').forEach(function (im) {
    var src = im.getAttribute('src') || im.getAttribute('data-src');
    var name = (im.getAttribute('alt') || im.getAttribute('title') || '').trim();
    var u = toAbs(src);
    if (!isJdImage(u) || !name) return;
    if (!out.sku_images[name]) {
      out.sku_images[name] = u;
      out.skus.push({ name: '', value: name, price: 0, stock: null, image: u });
    }
  });
  out.debug.skuCount = out.skus.length;

  // ───────────────────────────────────────────────────────────
  // 7. 详情图
  //    策略 1：#zbViewWeChatMiniImages 的 value（逗号分隔的移动端详情图）
  //    策略 2：详情容器内 <style> 标签里的 background-image:url()
  //    策略 3：详情容器内的 <img> 标签
  //    策略 4：getComputedStyle 兜底
  // ───────────────────────────────────────────────────────────
  var seenD = {};
  var dbg = {};

  // 策略 1
  var zbEl = document.getElementById('zbViewWeChatMiniImages');
  if (zbEl && zbEl.value) {
    zbEl.value.split(',').forEach(function (p) { pushImg(out.detail_images, seenD, p); });
  }
  dbg.afterZb = out.detail_images.length;

  // 策略 2：只取详情容器内部的 <style>，排除全局 CSS
  var styleNodes = qsa('style').filter(function (node) {
    var parent = node.parentElement;
    while (parent) {
      var pid = parent.id || '';
      var pcls = typeof parent.className === 'string' ? parent.className : '';
      if (pid.indexOf('detail-') === 0 || pid.indexOf('related-layout-') === 0 || pcls.indexOf('scoped') >= 0) return true;
      parent = parent.parentElement;
    }
    return false;
  });
  dbg.detailStyleCount = styleNodes.length;
  styleNodes.forEach(function (node) {
    var text = node.textContent || '';
    var needle = 'background-image:url(';
    var idx = 0;
    while (true) {
      idx = text.indexOf(needle, idx);
      if (idx === -1) break;
      idx += needle.length;
      var end = text.indexOf(')', idx);
      if (end === -1) break;
      pushImg(out.detail_images, seenD, text.substring(idx, end));
      idx = end + 1;
    }
  });
  dbg.afterStyle = out.detail_images.length;

  // 策略 3：详情容器内的 <img>
  var wrappers = qsa('[class*="scoped"], #detail-main, #detail-top, #detail, [id^="related-layout-"]');
  var imgCount = 0;
  wrappers.forEach(function (w) {
    Array.prototype.slice.call(w.querySelectorAll('img')).forEach(function (im) {
      imgCount++;
      pushImg(out.detail_images, seenD, im.getAttribute('src') || im.getAttribute('data-src') || im.getAttribute('data-lazy-img'));
    });
  });
  dbg.wrapperCount = wrappers.length;
  dbg.imgScanned = imgCount;
  dbg.afterImg = out.detail_images.length;

  // 策略 4
  if (!out.detail_images.length) {
    qsa('[class*="ssd-module"], .ssd-module-wrap .ssd-module').forEach(function (mod) {
      try {
        var bg = window.getComputedStyle(mod).backgroundImage;
        if (bg && bg !== 'none') {
          var m = bg.match(/url\(["']?([^"')]+)["']?\)/);
          if (m && m[1]) pushImg(out.detail_images, seenD, m[1]);
        }
      } catch (e) {}
    });
    dbg.computedStyleUsed = true;
  }

  out.debug.detail = dbg;
  out.debug.detailCount = out.detail_images.length;

  // ───────────────────────────────────────────────────────────
  // 8. 描述文本
  // ───────────────────────────────────────────────────────────
  var descEl = document.querySelector('#detail .detail-content, .detail-content, [class*="detailContent"]');
  if (descEl) out.description.text = (descEl.textContent || '').trim().substring(0, 5000);
  if (!out.description.text) {
    out.description.text = out.description.specs
      .map(function (s) { return s.key + ': ' + s.value; })
      .join('\n');
  }

  return out;
})()
