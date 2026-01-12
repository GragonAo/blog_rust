#!/bin/bash

# 微服务管理脚本

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_DIR="/home/gragon/work/rust/blog"
LOG_DIR="$PROJECT_DIR/logs"

# 确保日志目录存在
mkdir -p "$LOG_DIR"

# 启动服务
start_service() {
    local service=$1
    local port=$2
    
    if pgrep -f "target/debug/$service" > /dev/null; then
        echo -e "${YELLOW}⚠${NC}  $service 已经在运行"
        return
    fi
    
    cd "$PROJECT_DIR"
    ./target/debug/$service > /dev/null 2>&1 &
    sleep 2
    
    if pgrep -f "target/debug/$service" > /dev/null; then
        echo -e "${GREEN}✓${NC} $service 启动成功 (端口: $port)"
    else
        echo -e "${RED}✗${NC} $service 启动失败"
    fi
}

# 停止服务
stop_service() {
    local service=$1
    
    if pgrep -f "target/debug/$service" > /dev/null; then
        pkill -f "target/debug/$service"
        sleep 1
        echo -e "${GREEN}✓${NC} $service 已停止"
    else
        echo -e "${YELLOW}⚠${NC}  $service 未运行"
    fi
}

# 查看服务状态
status_service() {
    local service=$1
    local port=$2
    
    if pgrep -f "target/debug/$service" > /dev/null; then
        local pid=$(pgrep -f "target/debug/$service")
        echo -e "${GREEN}●${NC} $service ${GREEN}运行中${NC} (PID: $pid, 端口: $port)"
    else
        echo -e "${RED}●${NC} $service ${RED}未运行${NC}"
    fi
}

# 主菜单
case "$1" in
    start)
        echo -e "${BLUE}=== 启动所有服务 ===${NC}"
        start_service "user-service" "5010/50051"
        start_service "article-service" "5030"
        start_service "auth-service" "5020"
        start_service "gateway-service" "8080"
        ;;
    stop)
        echo -e "${BLUE}=== 停止所有服务 ===${NC}"
        stop_service "gateway-service"
        stop_service "auth-service"
        stop_service "article-service"
        stop_service "user-service"
        ;;
    restart)
        echo -e "${BLUE}=== 重启所有服务 ===${NC}"
        $0 stop
        sleep 2
        $0 start
        ;;
    status)
        echo -e "${BLUE}=== 服务状态 ===${NC}"
        status_service "gateway-service" "8080"
        status_service "user-service" "5010/50051"
        status_service "article-service" "5030"
        status_service "auth-service" "5020"
        echo ""
        echo "日志目录: $LOG_DIR/"
        ;;
    logs)
        exec "$PROJECT_DIR/view_logs.sh" "$2"
        ;;
    build)
        echo -e "${BLUE}=== 构建所有服务 ===${NC}"
        cd "$PROJECT_DIR"
        cargo build --bin user-service --bin article-service --bin auth-service --bin gateway-service
        ;;
    test)
        echo -e "${BLUE}=== 运行网关测试 ===${NC}"
        cd "$PROJECT_DIR/gateway-service"
        ./test_full_flow.sh
        ;;
    *)
        echo "微服务管理脚本"
        echo ""
        echo "用法: $0 {start|stop|restart|status|logs|build|test}"
        echo ""
        echo "命令:"
        echo "  start   - 启动所有服务"
        echo "  stop    - 停止所有服务"
        echo "  restart - 重启所有服务"
        echo "  status  - 查看服务状态"
        echo "  logs    - 查看日志 (可选: logs 1-8)"
        echo "  build   - 构建所有服务"
        echo "  test    - 运行网关测试"
        echo ""
        echo "示例:"
        echo "  $0 start           # 启动服务"
        echo "  $0 status          # 查看状态"
        echo "  $0 logs 1          # 查看最新日志"
        echo "  $0 logs 2          # 实时跟踪网关日志"
        exit 1
        ;;
esac
