#!/bin/bash

# 日志查看脚本

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

LOG_DIR="logs"
DATE=$(date +%Y-%m-%d)

echo -e "${BLUE}=== 微服务日志查看器 ===${NC}"
echo ""
echo "日志文件位置: $LOG_DIR/"
echo "日期: $DATE"
echo ""

# 显示菜单
echo "选择操作："
echo "  1) 查看所有服务最新日志"
echo "  2) 实时跟踪网关日志"
echo "  3) 实时跟踪用户服务日志"
echo "  4) 实时跟踪认证服务日志"
echo "  5) 实时跟踪所有服务日志"
echo "  6) 搜索日志关键词"
echo "  7) 查看错误日志"
echo "  8) 查看慢请求日志 (>100ms)"
echo ""

if [ -z "$1" ]; then
    read -p "请输入选项 (1-8): " CHOICE
else
    CHOICE=$1
fi

case $CHOICE in
    1)
        echo -e "${GREEN}=== Gateway Service (最新10条) ===${NC}"
        tail -n 10 "$LOG_DIR/gateway-service.log.$DATE" 2>/dev/null || echo "无日志文件"
        echo ""
        echo -e "${GREEN}=== User Service (最新10条) ===${NC}"
        tail -n 10 "$LOG_DIR/user-service.log.$DATE" 2>/dev/null || echo "无日志文件"
        echo ""
        echo -e "${GREEN}=== Auth Service (最新10条) ===${NC}"
        tail -n 10 "$LOG_DIR/auth-service.log.$DATE" 2>/dev/null || echo "无日志文件"
        ;;
    2)
        echo -e "${GREEN}实时跟踪网关日志 (Ctrl+C 停止)${NC}"
        tail -f "$LOG_DIR/gateway-service.log.$DATE"
        ;;
    3)
        echo -e "${GREEN}实时跟踪用户服务日志 (Ctrl+C 停止)${NC}"
        tail -f "$LOG_DIR/user-service.log.$DATE"
        ;;
    4)
        echo -e "${GREEN}实时跟踪认证服务日志 (Ctrl+C 停止)${NC}"
        tail -f "$LOG_DIR/auth-service.log.$DATE"
        ;;
    5)
        echo -e "${GREEN}实时跟踪所有服务日志 (Ctrl+C 停止)${NC}"
        tail -f "$LOG_DIR"/*.log.$DATE
        ;;
    6)
        read -p "输入搜索关键词: " KEYWORD
        echo -e "${GREEN}搜索结果：${NC}"
        grep -i "$KEYWORD" "$LOG_DIR"/*.log.$DATE 2>/dev/null | tail -20
        ;;
    7)
        echo -e "${YELLOW}=== 错误日志 (ERROR/WARN) ===${NC}"
        grep -E "ERROR|WARN" "$LOG_DIR"/*.log.$DATE 2>/dev/null | tail -30
        ;;
    8)
        echo -e "${YELLOW}=== 慢请求日志 (耗时 >100ms) ===${NC}"
        grep -E "duration_ms=[1-9][0-9]{2,}" "$LOG_DIR/gateway-service.log.$DATE" 2>/dev/null | tail -20
        ;;
    *)
        echo "无效选项"
        exit 1
        ;;
esac
