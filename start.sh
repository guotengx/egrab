#!/bin/bash
# EGrab - OpenCode 启动脚本
# 自动加载 .env 中的环境变量后启动 opencode

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"

if [ -f "$ENV_FILE" ]; then
  echo "[EGrab] 加载环境变量: $ENV_FILE"
  set -a
  source "$ENV_FILE"
  set +a
else
  echo "[EGrab] 警告: .env 文件不存在，请先 cp .env.example .env 并填入配置"
  exit 1
fi

echo "[EGrab] 代理配置: HTTPS_PROXY=$HTTPS_PROXY"
echo "[EGrab] 直连列表: NO_PROXY=$NO_PROXY"
echo "[EGrab] 启动 OpenCode..."
echo ""

exec opencode "$@"
