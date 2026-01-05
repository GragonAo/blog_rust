#!/bin/bash

echo "=== CORS 跨域测试 ==="
echo ""

GATEWAY="http://localhost:8080"

# 测试颜色
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}1. 测试预检请求（OPTIONS）${NC}"
echo "测试从 http://localhost:3000 发起的预检请求"
curl -i -X OPTIONS $GATEWAY/api/user/info/1 \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: GET" \
  -H "Access-Control-Request-Headers: Authorization"
echo ""

echo -e "${YELLOW}2. 测试实际请求的 CORS 头${NC}"
echo "测试从允许的源发起请求"
curl -i -X GET $GATEWAY/health \
  -H "Origin: http://localhost:3000"
echo ""

echo -e "${YELLOW}3. 测试不允许的源${NC}"
echo "测试从未配置的源发起请求（应该被拒绝或没有 CORS 头）"
curl -i -X GET $GATEWAY/health \
  -H "Origin: http://evil-site.com"
echo ""

echo -e "${YELLOW}4. 测试带凭证的请求${NC}"
echo "测试 credentials（cookies、authorization）"
curl -i -X GET $GATEWAY/api/auth/login \
  -H "Origin: http://localhost:3000" \
  -H "Cookie: session=test123"
echo ""

echo -e "${YELLOW}5. 检查暴露的响应头${NC}"
echo "验证 Access-Control-Expose-Headers"
curl -i -X GET $GATEWAY/health \
  -H "Origin: http://localhost:3000"
echo ""

echo "=== 测试完成 ==="
echo ""
echo -e "${GREEN}预期结果：${NC}"
echo "1. OPTIONS 请求应返回 200 和以下响应头："
echo "   - Access-Control-Allow-Origin: http://localhost:3000"
echo "   - Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS"
echo "   - Access-Control-Allow-Headers: authorization (或其他请求的头)"
echo "   - Access-Control-Allow-Credentials: true"
echo "   - Access-Control-Max-Age: 3600"
echo ""
echo "2. 实际请求应包含："
echo "   - Access-Control-Allow-Origin: http://localhost:3000"
echo "   - Access-Control-Expose-Headers: Content-Length, Content-Type"
echo "   - Access-Control-Allow-Credentials: true"
echo ""
echo "3. 不允许的源不应有 CORS 头或请求被拒绝"
