// EGrab - IPC Commands: 抓取规则包管理 + 页面诊断快照
//
// 这组命令让平台改版后的适配可以完全在磁盘上完成：
//   1. dump_page_snapshot  —— 把当前页面的真实结构导出，用于编写新规则
//   2. get_rules_info      —— 查看当前生效的规则来源/版本/平台
//   3. open_rules_folder   —— 打开规则目录直接编辑
//   4. reload_rules        —— 校验磁盘规则文件是否合法（规则本身每次抓取都会重新读取）

use crate::cdp::CdpManager;
use crate::models::{ErrorCode, IpcError};
use crate::parser::rules;

/// 当前生效的规则包信息。
#[tauri::command]
pub async fn get_rules_info() -> Result<serde_json::Value, IpcError> {
    let (pack, source) = rules::load_rule_pack();
    Ok(serde_json::json!({
        "source": source.source,
        "path": source.path,
        "rules_dir": rules::rules_dir().display().to_string(),
        "snapshots_dir": rules::snapshots_dir().display().to_string(),
        "version": pack.version,
        "updated_at": pack.updated_at,
        "error": source.error,
        "platforms": pack.platforms.iter().map(|p| serde_json::json!({
            "id": p.id,
            "label": p.label,
            "match_url": p.match_url,
            "extract_js_file": p.extract_js_file,
            "expand_js_file": p.expand_js_file,
            "has_extract_js": p.extract_js.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false),
        })).collect::<Vec<_>>(),
    }))
}

/// 重新校验磁盘上的规则文件。
///
/// 规则在每次抓取时都会重新从磁盘读取，因此本命令的作用是**提前发现语法错误**，
/// 而不是刷新缓存。
#[tauri::command]
pub async fn reload_rules() -> Result<serde_json::Value, IpcError> {
    let (_pack, source) = rules::load_rule_pack();
    if let Some(err) = source.error {
        return Err(IpcError {
            code: ErrorCode::ConfigInvalid,
            message: format!("规则文件无法解析，已回退到内置规则：{}", err),
            recoverable: true,
            step: None,
            details: Some(serde_json::json!({ "path": source.path })),
        });
    }
    get_rules_info().await
}

/// 打开规则目录（供用户直接编辑规则文件）。
#[tauri::command]
pub async fn open_rules_folder() -> Result<bool, IpcError> {
    let dir = rules::rules_dir();
    std::fs::create_dir_all(&dir).map_err(|e| IpcError {
        code: ErrorCode::StorageFailed,
        message: format!("无法创建规则目录 {}: {}", dir.display(), e),
        recoverable: true,
        step: None,
        details: None,
    })?;

    let result = {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(&dir).spawn()
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer").arg(&dir).spawn()
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            std::process::Command::new("xdg-open").arg(&dir).spawn()
        }
    };

    result.map_err(|e| IpcError {
        code: ErrorCode::StorageFailed,
        message: format!("无法打开规则目录: {}", e),
        recoverable: true,
        step: None,
        details: None,
    })?;

    Ok(true)
}

/// 导出当前浏览器页面的诊断快照。
///
/// 输出两个文件到 `<app_data>/rules/snapshots/`：
///   - `snapshot_{ts}.html`  完整 DOM（用于确认选择器）
///   - `snapshot_{ts}.json`  候选全局变量 + 图片清单 + 容器清单（用于确认取值路径）
///
/// 平台改版时，把这两个文件发给维护者即可写出新规则，全程无需重新编译。
#[tauri::command]
pub async fn dump_page_snapshot(cdp: tauri::State<'_, CdpManager>) -> Result<String, IpcError> {
    let dir = rules::snapshots_dir();
    std::fs::create_dir_all(&dir).map_err(|e| IpcError {
        code: ErrorCode::StorageFailed,
        message: format!("无法创建快照目录 {}: {}", dir.display(), e),
        recoverable: true,
        step: None,
        details: None,
    })?;

    // ---- 1. 采集结构化诊断信息 ----
    let probe_js = r#"
        (function () {
            function safe(fn) { try { return fn(); } catch (e) { return null; } }
            function clip(v, n) {
                try {
                    var s = JSON.stringify(v);
                    if (!s) return null;
                    return s.length > n ? s.slice(0, n) + '...[truncated]' : s;
                } catch (e) { return '[unserializable]'; }
            }

            var globalNames = [
                'pageConfig', '__INITIAL_DATA__', '__INITIAL_STATE__', '__NEXT_DATA__',
                'g_config', '__ICE_APP_CONTEXT__', '__NUXT__', '__APOLLO_STATE__',
                'window.__data__', 'Hub', '__STORE__', '__PRELOADED_STATE__'
            ];
            var globals = {};
            for (var i = 0; i < globalNames.length; i++) {
                var name = globalNames[i].replace('window.', '');
                var val = safe(function () { return window[name]; });
                globals[name] = (val === undefined || val === null)
                    ? null
                    : { type: typeof val, keys: safe(function () { return Object.keys(val).slice(0, 60); }), preview: clip(val, 20000) };
            }

            var imgs = [];
            var nodes = document.querySelectorAll('img');
            for (var j = 0; j < nodes.length && j < 400; j++) {
                var im = nodes[j];
                imgs.push({
                    src: im.getAttribute('src') || '',
                    dataSrc: im.getAttribute('data-src') || im.getAttribute('data-lazy-img') || '',
                    cls: (typeof im.className === 'string' ? im.className : ''),
                    parentCls: (im.parentElement && typeof im.parentElement.className === 'string' ? im.parentElement.className : ''),
                    parentId: (im.parentElement ? im.parentElement.id : ''),
                    w: im.naturalWidth || 0,
                    h: im.naturalHeight || 0
                });
            }

            // 页面上出现过的 id 和 class 清单，便于挑选稳定的子串选择器
            var ids = [];
            var classSet = {};
            var all = document.querySelectorAll('[id],[class]');
            for (var k = 0; k < all.length && k < 4000; k++) {
                var el = all[k];
                if (el.id) ids.push(el.id);
                var c = (typeof el.className === 'string') ? el.className : '';
                c.split(/\s+/).forEach(function (name) { if (name) classSet[name] = (classSet[name] || 0) + 1; });
            }

            return {
                url: location.href,
                title: document.title,
                globals: globals,
                images: imgs,
                imageTotal: nodes.length,
                ids: ids.slice(0, 500),
                classes: Object.keys(classSet).slice(0, 1200),
                scrollHeight: document.body ? document.body.scrollHeight : 0
            };
        })()
    "#;

    let probe = cdp.evaluate(probe_js).await.unwrap_or(serde_json::Value::Null);

    // ---- 2. 采集完整 DOM ----
    let html = cdp
        .evaluate("document.documentElement.outerHTML")
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    // ---- 3. 落盘 ----
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let json_path = dir.join(format!("snapshot_{}.json", ts));
    let html_path = dir.join(format!("snapshot_{}.html", ts));

    let payload = serde_json::json!({
        "captured_at_unix": ts,
        "probe": probe,
        "html_file": html_path.display().to_string(),
        "html_length": html.len(),
    });

    std::fs::write(
        &json_path,
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string()),
    )
    .map_err(|e| IpcError {
        code: ErrorCode::StorageFailed,
        message: format!("写入快照失败 {}: {}", json_path.display(), e),
        recoverable: true,
        step: None,
        details: None,
    })?;

    if !html.is_empty() {
        let _ = std::fs::write(&html_path, &html);
    }

    tracing::info!(path = %json_path.display(), html_len = html.len(), "Page snapshot saved");
    Ok(json_path.display().to_string())
}
