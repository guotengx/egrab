/*
 * EGrab 规则脚本 —— 京东 详情区就绪判定
 *
 * 程序滚动完页面之后、提取数据之前，每 0.5 秒执行一次本脚本，
 * 直到返回 true 或超时（超时时间见 rules.json 的 ready_timeout_ms）。
 *
 * 为什么需要：京东商品页的「商品介绍」「规格参数」是两个独立的懒加载楼层，
 * 只有滚进可视区域才会请求数据并渲染。2026-10 实测有商品抓取时这两个楼层
 * 没被触发，详情图和规格参数整段缺失。
 *
 * 返回 false 时，本脚本会顺带把页面往这两个楼层滚动，促使它们加载。
 *
 * ⚠️ 修改本文件后保存即生效，无需重新编译或重装程序。
 */
(function () {
  var tick = window.__egrabReadyTick = (window.__egrabReadyTick || 0) + 1;

  var detail = document.querySelector('#detail-main, #detail-top, #detail-footer, #detail');
  // 只认规格参数楼层本身（.attrs）。页面顶部的「商品亮点」.highlight-attrs 首屏就有，
  // 用 [class*="attrs"] 这类宽泛写法会在规格楼层还没加载时误判为已就绪。
  var specs = document.querySelector('.attrs .item, .parameter2 li');
  var text = document.body ? (document.body.innerText || document.body.textContent || '') : '';

  // 平台明确提示没有内容或加载失败时，不再等待。
  if (/暂无商品介绍|网络出小差了|楼层加载失败/.test(text)) return true;
  // 详情和规格都到了；少数商品没有规格楼层，详情出现后最多再等约 3 秒。
  if (detail && (specs || tick >= 6)) return true;

  // 还没就绪：从详情所在的标签区开始逐屏往下扫，到底后回到起点循环。
  var view = window.innerHeight || 800;
  var offset = window.pageYOffset || 0;
  var anchor = document.querySelector('.left-tabs-container, [class*="left-tabs"]');
  var top = 0;
  if (anchor && anchor.getBoundingClientRect) {
    top = Math.max(0, anchor.getBoundingClientRect().top + offset);
  }
  var maxY = Math.max(0, (document.body ? document.body.scrollHeight : 0) - view);
  var span = Math.max(view, maxY - top);
  var y = top + ((tick - 1) * Math.round(view * 0.8)) % span;
  window.scrollTo(0, Math.min(y, maxY));
  return false;
})()
