#!/bin/bash

echo "=== 网关功能测试 ==="
echo ""

GATEWAY="http://localhost:8080"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}1. 测试健康检查（白名单路径，无需JWT）${NC}"
curl -s -w "\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  $GATEWAY/health
echo ""

echo -e "${YELLOW}2. 测试登录接口（白名单路径，无需JWT）${NC}"
LOGIN_RESPONSE=$(curl -s -w "\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  -X POST $GATEWAY/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"address":"0x1234567890abcdef","chain":"evm"}')
echo "$LOGIN_RESPONSE"
echo ""

# 提取 token（假设响应包含 token 字段）
TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -n "$TOKEN" ]; then
  echo -e "${GREEN}✓ 获取到 Token: ${TOKEN:0:50}...${NC}"
else
  echo -e "${RED}✗ 未获取到 Token${NC}"
  TOKEN="fake-token-for-testing"
fi
echo ""

echo -e "${YELLOW}3. 测试无Token访问受保护接口（应该返回401）${NC}"
curl -s -w "\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  $GATEWAY/api/user/info/1
echo ""

echo -e "${YELLOW}4. 测试错误Token访问受保护接口（应该返回401）${NC}"
curl -s -w "\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  -H "Authorization: Bearer invalid-token-xyz" \
  $GATEWAY/api/user/info/1
echo ""

echo -e "${YELLOW}5. 测试有效Token访问受保护接口（应该返回200）${NC}"
curl -s -w "\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  -H "Authorization: Bearer $TOKEN" \
  $GATEWAY/api/user/info/1
echo ""

echo -e "${YELLOW}6. 测试限流（快速发送100个请求）${NC}"
echo "发送100个请求..."
success=0
rate_limited=0

for i in {1..100}; do
  http_code=$(curl -s -o /dev/null -w "%{http_code}" $GATEWAY/health)
  if [ "$http_code" == "200" ]; then
    ((success++))
  elif [ "$http_code" == "429" ]; then
    ((rate_limited++))
  fi
done

echo -e "${GREEN}成功: $success${NC}"
echo -e "${RED}被限流: $rate_limited${NC}"
echo ""

echo -e "${YELLOW}7. 测试慢请求（>1s会有警告日志）${NC}"
echo "（查看网关日志查看慢请求警告）"
echo ""

echo "=== 测试完成 ==="
echo ""
echo "网关日志应该显示："
echo "  - 请求路径和方法"
echo "  - 响应状态码"
echo "  - 每个请求的耗时（毫秒）"
echo "  - JWT验证日志（白名单路径会跳过）"
echo "  - 限流警告（如果触发）"
echo "  - 慢请求警告（>1s）"
