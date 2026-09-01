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

  // 商品图判定（比 isJdImage 更严格，用于主图和详情图）。
  //
  // 京东的商品图一律存放在 jfs/ 路径下；而页面上的图标、活动横幅、
  // 优惠券、店铺装修图走的是 imgzone / babel / da / cms 等路径。
  // 只认 jfs/ 能一次性滤掉绝大多数噪声。
  var NOISE_PATH = /\/(imgzone|imgtools|babel|cms|adver|coupon|da|jdcloud|activity|seckill|promo)\//i;
  function isJdProductImage(u) {
    if (!isJdImage(u)) return false;
    if (u.indexOf('jfs/') < 0) return false;
    if (NOISE_PATH.test(u)) return false;
    return true;
  }

  // 尺寸过滤：滤掉真正的小图标。
  // 注意不能一刀切用 naturalWidth —— 京东主图缩略图条是 s54x54_jfs，
  // 天然只有 54px，但它是商品图，清洗掉尺寸前缀后能拿到原图。
  // 所以：URL 带 sNxN_jfs 尺寸标记的一律放行，其余才按渲染尺寸判定。
  function passSizeGate(im, u) {
    if (/s\d+x\d+_jfs/i.test(u)) return true;
    var w = im && im.naturalWidth ? im.naturalWidth : 0;
    var h = im && im.naturalHeight ? im.naturalHeight : 0;
    if (w > 0 && h > 0 && (w < 150 || h < 150)) return false;
    return true;
  }

  // 判断元素是否落在"非商品"区域（推荐位、评价区、店铺栏、广告楼层）。
  var EXCLUDE_SEL = '[class*="recommend"],[id*="recommend"],[class*="comment"],[id*="comment"],' +
                    '[class*="rate"],[class*="shop"],[id*="shop"],[class*="advert"],[class*="banner"],' +
                    '[class*="guess"],[class*="hotsale"],[class*="rank"],[id*="footer"],[id*="header"]';
  function inExcludedZone(el) {
    try { return !!(el.closest && el.closest(EXCLUDE_SEL)); } catch (e) { return false; }
  }

  function pushImg(arr, seen, raw) {
    var u = toAbs(raw);
    if (!isJdImage(u)) return;
    var k = jdKey(u);
    if (seen[k]) return;
    seen[k] = true;
    arr.push(u);
  }

  // 严格版：只收商品图，并记录被拒样本用于下一轮诊断。
  var rejected = [];
  function pushProductImg(arr, seen, im, raw) {
    var u = toAbs(raw);
    if (!u) return;
    if (!isJdProductImage(u)) {
      if (rejected.length < 12) rejected.push({ why: 'not_product', u: u.slice(0, 140) });
      return;
    }
    if (!passSizeGate(im, u)) {
      if (rejected.length < 12) rejected.push({ why: 'too_small', u: u.slice(0, 140) });
      return;
    }
    if (im && inExcludedZone(im)) {
      if (rejected.length < 12) rejected.push({ why: 'excluded_zone', u: u.slice(0, 140) });
      return;
    }
    var k = jdKey(u);
    if (seen[k]) return;
    seen[k] = true;
    arr.push(u);
  }
  function imgSrc(im) {
    return im.getAttribute('src') || im.getAttribute('data-src') ||
           im.getAttribute('data-lazy-img') || im.getAttribute('data-origin') || '';
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
  var detailRoot = document.querySelector('#detail-main, #detail, #detail-top');

  // 分层选择器：从最精确到最宽泛，**第一组有产出就停**。
  // 上一版把所有选择器合并成一条，导致最宽泛的 carousel/preview
  // 把推荐位和活动图标一起捞进来（实测命中 35 张）。
  var galleryGroups = [
    ['#spec-list img', '#spec-n1 img', '#preview img'],
    ['[class*="gallery"] [class*="thumb"] img', '[class*="Gallery"] [class*="thumb"] img'],
    ['[class*="gallery"] img', '[class*="Gallery"] img'],
    ['[class*="mainImage"] img', '[class*="main-img"] img', '[class*="mainPic"] img'],
    ['[class*="carousel"] img', '[class*="Carousel"] img', '[class*="preview"] img']
  ];
  var groupUsed = -1;
  for (var gi = 0; gi < galleryGroups.length; gi++) {
    var nodes = qsa(galleryGroups[gi].join(','));
    if (!nodes.length) continue;
    nodes.forEach(function (im) {
      if (detailRoot && detailRoot.contains(im)) return; // 详情区的图不算主图
      pushProductImg(out.gallery, seenG, im, imgSrc(im));
    });
    if (out.gallery.length) { groupUsed = gi; break; }
  }
  out.debug.galleryGroupUsed = groupUsed;

  // 兜底：整页扫描带尺寸标记的商品图（排除详情区）
  if (!out.gallery.length) {
    qsa('img').forEach(function (im) {
      if (out.gallery.length >= 12) return;
      if (detailRoot && detailRoot.contains(im)) return;
      var src = imgSrc(im);
      if (!/\/n\d\/|s\d+x\d+_jfs/i.test(src)) return;
      pushProductImg(out.gallery, seenG, im, src);
    });
    out.debug.galleryFallbackUsed = true;
  }
  if (out.gallery.length > 12) out.gallery = out.gallery.slice(0, 12);

  // 封面：优先取主图区的大图，取不到再退回 gallery[0]
  var coverEl = document.querySelector('#spec-n1 img, [class*="mainImage"] img, [class*="bigImg"] img, [class*="big-img"] img');
  if (coverEl) {
    var coverUrl = toAbs(imgSrc(coverEl));
    if (isJdProductImage(coverUrl)) out.cover = coverUrl;
  }
  if (!out.cover) out.cover = out.gallery.length ? out.gallery[0] : '';

  out.debug.galleryCount = out.gallery.length;
  out.debug.gallerySample = out.gallery.slice(0, 12);
  out.debug.coverUrl = out.cover;

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
  // 京东规格区历经多版改版，这里覆盖新旧两类结构：
  //   老版：#choose-attrs > .li.p-choose > .dt(规格名) + .dd > .item[data-value]
  //   新版：[class*="specification"] / [class*="choose"] 容器内的 item
  var skuRoots = qsa('#choose-attrs, [id^="choose-attr"], [class*="p-choose"], [class*="specification"], [class*="choose-attr"], [class*="skuAttr"], [class*="sku-attr"]');
  out.debug.skuRootCount = skuRoots.length;

  skuRoots.forEach(function (root) {
    // 规格名（如"颜色"/"套餐"）
    var dt = root.querySelector('.dt, [class*="label"], [class*="title"], [class*="name"]');
    var propName = dt ? (dt.textContent || '').trim().replace(/[:：]\s*$/, '') : '';

    var items = Array.prototype.slice.call(
      root.querySelectorAll('.item, [data-value], [class*="item"], li, a')
    );
    items.forEach(function (it) {
      var value = (it.getAttribute && it.getAttribute('data-value')) || '';
      if (!value) {
        var inner = it.querySelector ? it.querySelector('i, span, b') : null;
        value = inner ? (inner.textContent || '').trim() : (it.textContent || '').trim();
      }
      value = String(value).trim();
      if (!value || value.length > 40) return;

      var im = it.querySelector ? it.querySelector('img') : null;
      var u = im ? toAbs(imgSrc(im)) : '';
      if (u && !isJdProductImage(u)) u = '';

      // 同一规格值只记一次
      var dup = out.skus.some(function (s) { return s.value === value; });
      if (dup) return;

      out.skus.push({ name: propName, value: value, price: 0, stock: null, image: u });
      if (u) out.sku_images[value] = u;
    });
  });

  // 兜底：只抓带 alt/title 的规格缩略图
  if (!out.skus.length) {
    qsa('[class*="sku"] img, [class*="specification"] img').forEach(function (im) {
      var name = (im.getAttribute('alt') || im.getAttribute('title') || '').trim();
      var u = toAbs(imgSrc(im));
      if (!name || !isJdProductImage(u)) return;
      if (out.sku_images[name]) return;
      out.sku_images[name] = u;
      out.skus.push({ name: '', value: name, price: 0, stock: null, image: u });
    });
    out.debug.skuFallbackUsed = true;
  }
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
  // 实测 wrapperCount=11 时会把"猜你喜欢/本店推荐"等楼层一起扫进来，
  // 因此排除推荐/评价/店铺区，并且只收 jfs/ 商品图。
  var wrappers = qsa('[class*="scoped"], #detail-main, #detail-top, #detail, [id^="related-layout-"]')
    .filter(function (w) { return !inExcludedZone(w); });
  var imgCount = 0;
  wrappers.forEach(function (w) {
    Array.prototype.slice.call(w.querySelectorAll('img')).forEach(function (im) {
      imgCount++;
      pushProductImg(out.detail_images, seenD, im, imgSrc(im));
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
  out.debug.detailSample = out.detail_images.slice(0, 8);
  // 被过滤掉的样本，用于下一轮判断过滤是否过严/过松
  out.debug.rejectedSample = rejected;

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
