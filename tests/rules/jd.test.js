// EGrab 京东规则离线回归（真实商品页）
//
// 运行：node tests/rules/jd.test.js
//
// fixtures 来自 2026-10-28 用户在浏览器里滚动加载完后另存的 MHTML：
//   14010763782    火山岩红茶（单款式；3 张详情图由店铺编辑器插入，其中 1 张是产品信息/标签图）
//   14010613427    火山岩绿茶（单款式；产品信息/标签图由店铺编辑器插入）
//   10237705257646 有机水满香（3 个款式；17 张详情图全部由店铺编辑器插入）
// 店铺编辑器插入的图带 class="shop-editor-zone-insert"，v4 及以前会被 [class*="shop"] 排除区误杀。

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadPage, runScript } from './dom.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RULES_DIR = path.join(__dirname, '..', '..', 'src-tauri', 'rules');
const EXTRACT = path.join(RULES_DIR, 'jd.extract.js');
const READY = path.join(RULES_DIR, 'jd.ready.js');
const FIXTURES = path.join(__dirname, 'fixtures');

let passed = 0;
let failed = 0;
function check(name, cond, detail) {
  if (cond) {
    passed++;
    console.log('  PASS ' + name);
  } else {
    failed++;
    console.log('  FAIL ' + name + (detail === undefined ? '' : ' | ' + JSON.stringify(detail)));
  }
}

function page(sku) {
  return loadPage(path.join(FIXTURES, 'jd_' + sku + '.dom.json.gz'), 'https://item.jd.com/' + sku + '.html');
}

// 图片文件名去扩展名后的哈希部分，例如 .../0568b9b2e65a8e87.jpg.avif → 0568b9b2e65a8e87
function hash(url) {
  return String(url || '').split('?')[0].split('/').pop().split('.')[0];
}

// 所有 fixture 共用：#detail-main 里的每一张图都必须被收进 detail_images
function checkDetailComplete(p, out) {
  const inDom = p.document.querySelectorAll('#detail-main img').map((im) => hash(im.getAttribute('src')));
  const got = out.detail_images.map(hash);
  const missing = inDom.filter((h) => got.indexOf(h) < 0);
  check('#detail-main 内 ' + inDom.length + ' 张图全部收下', missing.length === 0, missing);
  const editorImgs = p.document.querySelectorAll('#detail-main img.shop-editor-zone-insert').length;
  const rejectedZone = (out.debug.rejectedSample || []).filter((r) => r.why === 'excluded_zone');
  check('店铺编辑器插入图 ' + editorImgs + ' 张无一被排除区误杀', rejectedZone.length === 0, rejectedZone);
}

console.log('== 火山岩红茶 14010763782（单款式，标签图被 v4 误杀）');
{
  const p = page('14010763782');
  const out = runScript(p, EXTRACT);
  const detail = out.detail_images.map(hash);
  const firstThumb = hash(p.document.querySelector('.thumbnails img.image').getAttribute('src'));
  const bigImage = hash(p.document.getElementById('spec-img').getAttribute('src'));
  check('标题', /火山岩正宗红茶/.test(out.title), out.title);
  check('主图 5 张且无图标', out.gallery.length === 5 && out.gallery.every((u) => u.indexOf('imagetools') < 0), out.gallery);
  check('封面取第 1 张缩略图（大图停在别的图上也不跟着变）',
    hash(out.cover) === firstThumb && firstThumb !== bigImage, { cover: hash(out.cover), firstThumb, bigImage });
  check('价格 57.8', out.price.min === 57.8 && out.price.max === 57.8, out.price);
  check('店铺名', out.shop.name === '椰仙旗舰店', out.shop.name);
  check('店铺链接', out.shop.url === 'https://mall.jd.com/index-111002.html', out.shop.url);
  check('规格参数 7 条', out.description.specs.length === 7, out.description.specs);
  check('详情图 9 张', detail.length === 9, detail);
  check('3 张编辑器插入图全部抓到',
    ['2c568287f85b9e0a', '6efa1d9f58e85402', '0568b9b2e65a8e87'].every((h) => detail.indexOf(h) >= 0), detail);
  check('产品信息/标签图是第 8 张（detail_008）', detail.indexOf('0568b9b2e65a8e87') === 7, detail);
  checkDetailComplete(p, out);
  check('无款式', out.skus.length === 0 && out.debug.skuSelected === '', out.skus);
  check('诊断：脚本版本 6、详情区块已渲染、无失败提示',
    out.debug.scriptVersion === 6 && out.debug.page.blocks.indexOf('detail-main') >= 0 && out.debug.page.failText === '',
    out.debug.page);
}

console.log('== 火山岩绿茶 14010613427（单款式，抓取时详情楼层没加载）');
{
  const p = page('14010613427');
  const out = runScript(p, EXTRACT);
  const detail = out.detail_images.map(hash);
  const firstThumb = hash(p.document.querySelector('.thumbnails img.image').getAttribute('src'));
  check('标题', /火山岩特色绿茶/.test(out.title), out.title);
  check('封面取第 1 张缩略图', hash(out.cover) === firstThumb, { cover: hash(out.cover), firstThumb });
  check('价格 57.8', out.price.min === 57.8, out.price);
  check('规格参数 6 条', out.description.specs.length === 6, out.description.specs);
  check('详情图 8 张', detail.length === 8, detail);
  check('产品信息/标签图是第 7 张（detail_007）', detail.indexOf('40c7dbc4c8d62049') === 6, detail);
  checkDetailComplete(p, out);
}

console.log('== 有机水满香 10237705257646（3 个款式，详情全部由编辑器插入）');
{
  const p = page('10237705257646');
  const out = runScript(p, EXTRACT);
  const values = out.skus.map((s) => s.value);
  check('款式 3 个且名称正确',
    JSON.stringify(values) === JSON.stringify(['五指山有机红茶礼盒80g', '五指山有机绿茶礼盒80g', '澄迈苦丁茶嫩芽礼盒80g']), values);
  check('款式规格名为「规格」且都有图', out.skus.every((s) => s.name === '规格' && /jfs\//.test(s.image)), out.skus);
  check('款式图 3 张', Object.keys(out.sku_images).length === 3, Object.keys(out.sku_images));
  check('记录当前选中款式', out.debug.skuSelected === '五指山有机红茶礼盒80g', out.debug.skuSelected);
  check('封面取选中款式的图', hash(out.cover) === '0083320320386654', out.cover);
  check('价格 459', out.price.min === 459, out.price);
  check('规格参数 8 条', out.description.specs.length === 8, out.description.specs);
  check('详情图 17 张（v4 为 0 张）', out.detail_images.length === 17, out.detail_images.length);
  checkDetailComplete(p, out);
}

console.log('== 就绪判定 jd.ready.js');
{
  const p = page('14010763782');
  const ready = runScript(p, READY);
  check('楼层都已渲染：立即返回 true，不滚动', ready === true && p.window.scrolls.length === 0, p.window.scrolls);
}
{
  // 模拟抓取时的现场：商品介绍、规格参数两个楼层都没渲染
  const p = page('14010613427');
  p.document.getElementById('detail-main').remove();
  p.document.querySelector('.attrs').remove();
  const specsLeft = p.document.querySelector('.attrs .item, .parameter2 li');
  check('（前置）规格与详情已移除', specsLeft === null && p.document.getElementById('detail-main') === null);
  const r1 = runScript(p, READY);
  const r2 = runScript(p, READY);
  check('楼层未渲染：返回 false 并滚动页面去触发懒加载',
    r1 === false && r2 === false && p.window.scrolls.length === 2, p.window.scrolls);
  p.document.body.appendText('暂无商品介绍');
  check('页面提示「暂无商品介绍」：不再等待', runScript(p, READY) === true);
}
{
  // 详情到了但没有规格楼层：最多再等几轮就放行，不卡满超时
  const p = page('14010763782');
  p.document.querySelector('.attrs').remove();
  const results = [];
  for (let i = 0; i < 6; i++) results.push(runScript(p, READY));
  check('只缺规格楼层：前 5 次等待、第 6 次放行',
    results.slice(0, 5).every((r) => r === false) && results[5] === true, results);
}

console.log('\n结果: ' + passed + ' PASS / ' + failed + ' FAIL');
process.exit(failed ? 1 : 0);
