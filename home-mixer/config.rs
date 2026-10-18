use anyhow::{bail, Context};

pub const ENV_GRPC_MTLS_ENABLED: &str = "GRPC_MTLS_ENABLED";
pub const ENV_GRPC_MTLS_SERVER_KEY_PATH: &str = "GRPC_MTLS_SERVER_KEY_PATH";
pub const ENV_GRPC_MTLS_SERVER_CRT_PATH: &str = "GRPC_MTLS_SERVER_CRT_PATH";
pub const ENV_GRPC_MTLS_SERVER_CHAIN_PATH: &str = "GRPC_MTLS_SERVER_CHAIN_PATH";
pub const ENV_GRPC_MTLS_CLIENT_CA_PATH: &str = "GRPC_MTLS_CLIENT_CA_PATH";

#[derive(Debug, Clone)]
pub struct GrpcMtlsConfig {
    pub server_key_path: String,
    pub server_crt_path: String,
    pub server_chain_path: Option<String>,
    pub client_ca_path: String,
}
impl GrpcMtlsConfig {
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        let enabled = std::env::var(ENV_GRPC_MTLS_ENABLED)
            .ok()
            .filter(|v| !v.is_empty())
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);

        if !enabled {
            return Ok(None);
        }

        let server_key_path = match std::env::var(ENV_GRPC_MTLS_SERVER_KEY_PATH)
            .ok()
            .filter(|v| !v.is_empty())
        {
            Some(v) => v,
            None => bail!(
                "{ENV_GRPC_MTLS_SERVER_KEY_PATH} must be set when {ENV_GRPC_MTLS_ENABLED}=true"
            ),
        };

        let server_crt_path = match std::env::var(ENV_GRPC_MTLS_SERVER_CRT_PATH)
            .ok()
            .filter(|v| !v.is_empty())
        {
            Some(v) => v,
            None => bail!(
                "{ENV_GRPC_MTLS_SERVER_CRT_PATH} must be set when {ENV_GRPC_MTLS_ENABLED}=true"
            ),
        };

        let server_chain_path = std::env::var(ENV_GRPC_MTLS_SERVER_CHAIN_PATH)
            .ok()
            .filter(|v| !v.is_empty());

        let client_ca_path = match std::env::var(ENV_GRPC_MTLS_CLIENT_CA_PATH)
            .ok()
            .filter(|v| !v.is_empty())
        {
            Some(v) => v,
            None => bail!(
                "{ENV_GRPC_MTLS_CLIENT_CA_PATH} must be set when {ENV_GRPC_MTLS_ENABLED}=true"
            ),
        };

        Ok(Some(Self {
            server_key_path,
            server_crt_path,
            server_chain_path,
            client_ca_path,
        }))
    }

    pub fn server_tls_config(&self) -> anyhow::Result<tonic::transport::ServerTlsConfig> {
        let mut cert_pem = std::fs::read(&self.server_crt_path)
            .with_context(|| format!("failed to read {}", self.server_crt_path))?;
        let key_pem = std::fs::read(&self.server_key_path)
            .with_context(|| format!("failed to read {}", self.server_key_path))?;
        let client_ca_pem = std::fs::read(&self.client_ca_path)
            .with_context(|| format!("failed to read {}", self.client_ca_path))?;

        if let Some(chain_path) = self.server_chain_path.as_ref() {
            let chain_pem = std::fs::read(chain_path)
                .with_context(|| format!("failed to read {}", chain_path))?;
            if !cert_pem.ends_with(b"\n") {
                cert_pem.push(b'\n');
            }
            cert_pem.extend_from_slice(&chain_pem);
        }

        let identity = tonic::transport::Identity::from_pem(cert_pem, key_pem);
        let client_ca = tonic::transport::Certificate::from_pem(client_ca_pem);

        Ok(tonic::transport::ServerTlsConfig::new()
            .identity(identity)
            .client_ca_root(client_ca))
    }
}
