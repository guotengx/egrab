#!/usr/bin/env bash
# EGrab 全局异步执行脚本 - 物理级防阻塞外挂（v2 - 带可观测性）
# 用法: ./async_run.sh "你的长耗时命令" "日志文件名"
#
# 设计原则：
# 1. 所有长耗时命令必须通过此脚本执行，禁止直接在终端运行
# 2. 脚本自动后台执行 + 5秒初始日志抽查
# 3. 生成 .status 和 .pid 文件用于判定命令状态
# 4. 判定规则：STATE=FINISHED && EXIT_CODE=0 => 成功；否则查看日志
set -u

COMMAND="${1:-}"
LOGFILE="${2:-}"

if [ -z "$COMMAND" ] || [ -z "$LOGFILE" ]; then
  echo "Usage: ./async_run.sh \"command\" \"logfile\""
  exit 2
fi

STATUS_FILE="${LOGFILE}.status"
PID_FILE="${LOGFILE}.pid"

echo "🚀 开始后台执行: $COMMAND"
echo "📄 日志文件: $LOGFILE"
echo "📌 状态文件: $STATUS_FILE"
echo "📌 PID 文件: $PID_FILE"

{
  echo "COMMAND=$COMMAND"
  echo "STARTED_AT=$(date '+%Y-%m-%d %H:%M:%S')"
  echo "STATE=RUNNING"
  echo "EXIT_CODE="
  echo "FINISHED_AT="
} > "$STATUS_FILE"

nohup bash -lc "
  set +e
  $COMMAND
  CODE=\$?
  STARTED_AT_VALUE=\$(grep '^STARTED_AT=' \"$STATUS_FILE\" | cut -d= -f2-)
  {
    echo \"COMMAND=$COMMAND\"
    echo \"STARTED_AT=\$STARTED_AT_VALUE\"
    echo \"STATE=FINISHED\"
    echo \"EXIT_CODE=\$CODE\"
    echo \"FINISHED_AT=\$(date '+%Y-%m-%d %H:%M:%S')\"
  } > \"$STATUS_FILE\"
  exit \$CODE
" > "$LOGFILE" 2>&1 &

PID=$!
echo "$PID" > "$PID_FILE"

echo "✅ 进程已挂载，PID: $PID"
echo "⏳ 正在抽查前 5 秒初始日志..."
sleep 5

echo "-----------------------------------"
tail -n 20 "$LOGFILE" 2>/dev/null || true
echo "-----------------------------------"

echo "📌 当前状态："
cat "$STATUS_FILE" 2>/dev/null || true

echo ""
echo "💡 后续必须执行："
echo "  cat $STATUS_FILE"
echo "  tail -n 50 $LOGFILE"
echo "  ps -p $PID -o pid,stat,etime,command"
echo ""
echo "判定规则："
echo "  STATE=FINISHED 且 EXIT_CODE=0 => 成功"
echo "  STATE=FINISHED 且 EXIT_CODE!=0 => 失败，查看日志"
echo "  STATE=RUNNING => 仍在运行"
echo "  无法判断 => 最多检查 2 次后升级给上级或 QA"
