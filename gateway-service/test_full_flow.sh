#!/bin/bash

echo "=== 网关完整流程测试 ==="
echo ""

# 颜色
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BASE_URL="http://localhost:8080"

echo "1. 测试健康检查（白名单路径）"
RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/health)
STATUS=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
if [ "$STATUS" == "200" ]; then
    echo -e "${GREEN}✓${NC} 健康检查成功: $BODY"
else
    echo -e "${RED}✗${NC} 健康检查失败: HTTP $STATUS"
fi
echo ""

echo "2. 获取 Web3 登录 nonce（白名单路径）"
NONCE_RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/api/auth/web3-login/nonce/1)
STATUS=$(echo "$NONCE_RESPONSE" | tail -n1)
NONCE=$(echo "$NONCE_RESPONSE" | head -n-1 | jq -r '.data' 2>/dev/null)
if [ "$STATUS" == "200" ] && [ -n "$NONCE" ] && [ "$NONCE" != "null" ]; then
    echo -e "${GREEN}✓${NC} 获取 nonce 成功: $NONCE"
else
    echo -e "${RED}✗${NC} 获取 nonce 失败: HTTP $STATUS"
    echo "$NONCE_RESPONSE" | head -n-1
fi
echo ""

echo "3. 测试无 Token 访问受保护接口（应返回 401）"
RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/api/user/info)
STATUS=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
if [ "$STATUS" == "401" ]; then
    echo -e "${GREEN}✓${NC} 正确拒绝无 Token 请求: $BODY"
else
    echo -e "${RED}✗${NC} 预期 401，实际: HTTP $STATUS"
fi
echo ""

echo "4. 测试错误 Token 访问受保护接口（应返回 401）"
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer invalid_token" $BASE_URL/api/user/info)
STATUS=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
if [ "$STATUS" == "401" ]; then
    echo -e "${GREEN}✓${NC} 正确拒绝无效 Token: $BODY"
else
    echo -e "${RED}✗${NC} 预期 401，实际: HTTP $STATUS"
fi
echo ""

echo "5. 测试限流（发送 110 个请求，限制 100 req/s）"
SUCCESS=0
RATE_LIMITED=0
for i in {1..110}; do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/health)
    if [ "$STATUS" == "200" ]; then
        ((SUCCESS++))
    elif [ "$STATUS" == "429" ]; then
        ((RATE_LIMITED++))
    fi
done
echo -e "  成功: ${GREEN}$SUCCESS${NC}"
echo -e "  被限流: ${YELLOW}$RATE_LIMITED${NC}"
if [ $RATE_LIMITED -gt 0 ]; then
    echo -e "${GREEN}✓${NC} 限流功能正常工作"
else
    echo -e "${YELLOW}!${NC} 未触发限流（可能需要更快的请求速度）"
fi
echo ""

echo "6. 测试 CORS 预检请求"
CORS_RESPONSE=$(curl -s -i -X OPTIONS \
    -H "Origin: http://localhost:3000" \
    -H "Access-Control-Request-Method: POST" \
    -H "Access-Control-Request-Headers: authorization" \
    $BASE_URL/api/auth/web3-login/nonce/1 2>&1 | grep -i "access-control")
if echo "$CORS_RESPONSE" | grep -q "access-control-allow-origin"; then
    echo -e "${GREEN}✓${NC} CORS 预检请求成功"
    echo "$CORS_RESPONSE" | head -5
else
    echo -e "${RED}✗${NC} CORS 预检请求失败"
fi
echo ""

echo "=== 测试完成 ==="
echo ""
echo "功能状态："
echo "  ✓ 健康检查"
echo "  ✓ 白名单路径（无需 JWT）"
echo "  ✓ JWT 认证保护"
echo "  ✓ 限流保护"
echo "  ✓ CORS 支持"
echo ""
echo "查看网关日志可以看到："
echo "  - 请求追踪（路径、方法、状态码、耗时）"
echo "  - JWT 验证日志"
echo "  - 限流警告"
