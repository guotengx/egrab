/*
 * EGrab 规则脚本 —— 京东 详情区展开（在滚动触发懒加载之前执行）
 *
 * 京东详情容器使用 固定 height + overflow:hidden + transform:scale，
 * 会让 window.scrollTo 永远触达不到真正的图片元素，导致详情图抓不全。
 * 这里先强制展开这些容器，并把图片改为 eager 加载。
 *
 * 使用 [class*="scoped"] 子串匹配，避免依赖京东的构建哈希类名。
 */
(function () {
  var touched = 0;

  // 1. 已知的详情容器 ID
  ['detail-main', 'detail-top', 'detail', 'related-layout-head', 'related-layout-footer', 'event-zone']
    .forEach(function (id) {
      var el = document.getElementById(id);
      if (el) {
        el.style.height = 'auto';
        el.style.maxHeight = 'none';
        el.style.overflow = 'visible';
        touched++;
      }
    });

  // 2. scoped 详情根容器及其所有后代
  var roots = document.querySelectorAll('[class*="scoped"], [id^="detail-"], [id^="related-layout-"]');
  Array.prototype.slice.call(roots).forEach(function (root) {
    root.style.height = 'auto';
    root.style.maxHeight = 'none';
    root.style.overflow = 'visible';
    Array.prototype.slice.call(root.querySelectorAll('*')).forEach(function (c) {
      var s = c.style;
      if (!s) return;
      if (s.overflow === 'hidden') s.overflow = 'visible';
      var h = s.height || s.maxHeight;
      if (h && h !== 'auto') {
        s.height = 'auto';
        s.maxHeight = 'none';
      }
      touched++;
    });
    // 3. 图片改为立即加载
    Array.prototype.slice.call(root.querySelectorAll('img')).forEach(function (img) {
      img.loading = 'eager';
      img.decoding = 'sync';
      var lazy = img.getAttribute('data-lazy-img') || img.getAttribute('data-src');
      if (lazy && !img.getAttribute('src')) img.setAttribute('src', lazy);
    });
  });

  return 'expanded:' + touched;
})()
