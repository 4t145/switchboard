use std::path::PathBuf;

/// Unix 系统中查找本地 TLS 证书的标准路径
pub struct UnixTlsSearchPaths {
    /// 系统 CA 证书
    pub system_ca_paths: Vec<PathBuf>,
    /// 用户证书路径
    pub user_cert_paths: Vec<PathBuf>,
    /// 应用证书路径
    pub app_cert_paths: Vec<PathBuf>,
}

impl UnixTlsSearchPaths {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir();
        let config_dir = dirs::config_dir();
        let mut user_cert_paths = Vec::new();
        if let Some(home) = &home_dir {
            user_cert_paths.push(home.join(".ssl"));
            user_cert_paths.push(home.join(".certs"));
        };
        if let Some(config) = &config_dir {
            user_cert_paths.push(config.join("switchboard/certs"));
        }
        Self {
            system_ca_paths: vec![
                // Debian/Ubuntu
                PathBuf::from("/etc/ssl/certs"),
                PathBuf::from("/usr/share/ca-certificates"),
                // RedHat/CentOS
                PathBuf::from("/etc/pki/tls/certs"),
                PathBuf::from("/etc/pki/ca-trust/source/anchors"),
                // Alpine
                PathBuf::from("/etc/ssl/certs"),
            ],
            user_cert_paths,
            app_cert_paths: vec![
                // Switchboard 特定路径
                PathBuf::from("/etc/switchboard/certs"),
                // Let's Encrypt
                PathBuf::from("/etc/letsencrypt/live"),
                // 常见应用位置
                PathBuf::from("/etc/ssl/private"),
                PathBuf::from("/etc/pki/tls/private"),
            ],
        }
    }

    /// 搜索证书文件
    pub async fn search_certificates(&self, domain: &str) -> Vec<CertificateLocation> {
        let mut results = Vec::new();

        // 搜索 Let's Encrypt
        let le_path = PathBuf::from("/etc/letsencrypt/live").join(domain);
        if le_path.exists() {
            results.push(CertificateLocation {
                cert_path: le_path.join("fullchain.pem"),
                key_path: le_path.join("privkey.pem"),
                source: "letsencrypt".to_string(),
            });
        }

        // 搜索 Switchboard 目录
        for base_path in &self.app_cert_paths {
            let cert = base_path.join(format!("{}.crt", domain));
            let key = base_path.join(format!("{}.key", domain));

            if cert.exists() && key.exists() {
                results.push(CertificateLocation {
                    cert_path: cert,
                    key_path: key,
                    source: "local".to_string(),
                });
            }
        }

        results
    }
}

#[derive(Debug, Clone)]
pub struct CertificateLocation {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub source: String,
}

/// 检测系统类型并返回对应的路径
pub fn detect_system_cert_paths() -> Vec<PathBuf> {
    let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();

    if os_release.contains("Ubuntu") || os_release.contains("Debian") {
        vec![
            PathBuf::from("/etc/ssl/certs"),
            PathBuf::from("/etc/ssl/private"),
        ]
    } else if os_release.contains("CentOS") || os_release.contains("Red Hat") {
        vec![
            PathBuf::from("/etc/pki/tls/certs"),
            PathBuf::from("/etc/pki/tls/private"),
        ]
    } else if os_release.contains("Alpine") {
        vec![
            PathBuf::from("/etc/ssl/certs"),
            PathBuf::from("/etc/ssl/private"),
        ]
    } else {
        // 默认位置
        vec![
            PathBuf::from("/etc/ssl/certs"),
            PathBuf::from("/etc/ssl/private"),
        ]
    }
}
