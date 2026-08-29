/*
 * EGrab 规则脚本 —— 淘宝 / 天猫 详情区展开（在滚动触发懒加载之前执行）
 *
 * 天猫图文详情多为懒加载 <img data-src>，滚动前先把 data-src 提升为 src，
 * 并解除详情容器的高度限制，最大化详情图命中率。
 */
(function () {
  var touched = 0;

  var roots = document.querySelectorAll(
    '#imageTextInfo-container, [class*="descV8"], [class*="desc-root"], #description, [id*="detail"]'
  );
  Array.prototype.slice.call(roots).forEach(function (root) {
    root.style.height = 'auto';
    root.style.maxHeight = 'none';
    root.style.overflow = 'visible';
    Array.prototype.slice.call(root.querySelectorAll('img')).forEach(function (img) {
      img.loading = 'eager';
      img.decoding = 'sync';
      var lazy = img.getAttribute('data-src') || img.getAttribute('data-lazyload-src') || img.getAttribute('data-ks-lazyload');
      if (lazy && !img.getAttribute('src')) img.setAttribute('src', lazy);
      touched++;
    });
  });

  return 'expanded:' + touched;
})()
