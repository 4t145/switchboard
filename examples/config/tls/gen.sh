#!/bin/bash
set -e

cd "$(dirname "$0")"

# 1. 生成 CA
# CA 自身通常不需要复杂的 SAN，保持简单即可
echo "Generating CA..."
openssl genrsa -out ca.key 2048
openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 -out ca.crt -subj "/CN=Switchboard CA"

# 定义通用生成函数
gen_cert() {
    local name=$1
    local domain=$2
    
    echo "Generating Certificate for $domain ($name)..."
    
    # 1. 生成私钥
    openssl genrsa -out "$name.key" 2048
    
    # 2. 生成 CSR (关键步骤)
    # 使用 -addext 直接在 CSR 中声明这是 v3 证书需要的扩展
    # subjectAltName 是 rustls 必须的
    openssl req -new -key "$name.key" -out "$name.csr" -subj "/CN=$domain" \
        -addext "subjectAltName=DNS:$domain" \
        -addext "basicConstraints=CA:FALSE" \
        -addext "keyUsage=digitalSignature,keyEncipherment" \
        -addext "extendedKeyUsage=serverAuth"

    # 3. 签名
    # 使用 -copy_extensions copy 将 CSR 里的扩展抄写到证书里
    openssl x509 -req -in "$name.csr" -CA ca.crt -CAkey ca.key -CAcreateserial \
        -out "$name.crt" -days 365 -sha256 \
        -copy_extensions copy
        
    rm "$name.csr"
}

# 生成证书
gen_cert "single" "single.test.local"
gen_cert "sni-a" "sni-a.test.local"
gen_cert "sni-b" "sni-b.test.local"

echo "Cleaning up..."
rm -f ca.srl

echo "Done. All certificates are X.509 v3 with SAN."