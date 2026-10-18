// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::{fs, io};
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint, Identity, ServerTlsConfig};

pub const ENV_TLS_CA: &str = "COPY_PORT_TLS_CA";
pub const ENV_TLS_CERT: &str = "COPY_PORT_TLS_CERT";
pub const ENV_TLS_KEY: &str = "COPY_PORT_TLS_KEY";
pub const ENV_TLS_SERVER_NAME: &str = "COPY_PORT_TLS_SERVER_NAME";

fn install_crypto_provider() {}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, msg)
}

fn read(path: &str, what: &str) -> io::Result<Vec<u8>> {
    fs::read(path).map_err(|e| io::Error::new(e.kind(), format!("reading {what} {path}: {e}")))
}

#[derive(Debug, Clone)]
pub struct ServerTlsOptions {
    pub cert_path: String,
    pub key_path: String,
    pub client_ca_path: Option<String>,
}

impl ServerTlsOptions {
    pub fn from_paths(
        cert_path: Option<String>,
        key_path: Option<String>,
        client_ca_path: Option<String>,
    ) -> io::Result<Option<Self>> {
        let cert_path = cert_path.filter(|s| !s.is_empty());
        let key_path = key_path.filter(|s| !s.is_empty());
        let client_ca_path = client_ca_path.filter(|s| !s.is_empty());
        match (cert_path, key_path) {
            (Some(cert_path), Some(key_path)) => Ok(Some(Self {
                cert_path,
                key_path,
                client_ca_path,
            })),
            (None, None) if client_ca_path.is_some() => Err(invalid(
                "tls client CA requires tls cert and key".to_string(),
            )),
            (None, None) => Ok(None),
            _ => Err(invalid("tls cert and key must be set together".to_string())),
        }
    }

    pub fn load(&self) -> io::Result<ServerTlsConfig> {
        install_crypto_provider();
        let identity = Identity::from_pem(
            read(&self.cert_path, "tls cert")?,
            read(&self.key_path, "tls key")?,
        );
        let mut tls = ServerTlsConfig::new().identity(identity);
        if let Some(ca) = &self.client_ca_path {
            tls = tls.client_ca_root(Certificate::from_pem(read(ca, "tls client CA")?));
        }
        log::info!(
            "copy_port server TLS enabled (client certs required: {})",
            self.client_ca_path.is_some()
        );
        Ok(tls)
    }
}

#[derive(Debug, Clone)]
pub struct ClientTlsOptions {
    pub ca_path: String,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub server_name: Option<String>,
}

impl ClientTlsOptions {
    pub fn from_env() -> io::Result<Option<Self>> {
        let get = |k: &str| std::env::var(k).ok().filter(|s| !s.is_empty());
        let (ca_path, cert_path, key_path, server_name) = (
            get(ENV_TLS_CA),
            get(ENV_TLS_CERT),
            get(ENV_TLS_KEY),
            get(ENV_TLS_SERVER_NAME),
        );
        let Some(ca_path) = ca_path else {
            if cert_path.is_some() || key_path.is_some() || server_name.is_some() {
                return Err(invalid(format!("COPY_PORT_TLS_* set without {ENV_TLS_CA}")));
            }
            return Ok(None);
        };
        if cert_path.is_some() != key_path.is_some() {
            return Err(invalid(format!(
                "{ENV_TLS_CERT} and {ENV_TLS_KEY} must be set together"
            )));
        }
        Ok(Some(Self {
            ca_path,
            cert_path,
            key_path,
            server_name,
        }))
    }

    pub fn load(&self) -> io::Result<ClientTlsConfig> {
        install_crypto_provider();
        let mut tls = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(read(&self.ca_path, "tls CA")?));
        if let (Some(cert), Some(key)) = (&self.cert_path, &self.key_path) {
            tls = tls.identity(Identity::from_pem(
                read(cert, "tls cert")?,
                read(key, "tls key")?,
            ));
        }
        if let Some(name) = &self.server_name {
            tls = tls.domain_name(name.clone());
        }
        Ok(tls)
    }
}

pub fn endpoint_for(url: &str, tls: Option<&ClientTlsOptions>) -> io::Result<Endpoint> {
    let endpoint = Endpoint::from_shared(url.to_string())
        .map_err(|e| invalid(format!("bad copy_port url {url}: {e}")))?;
    if !url.starts_with("https://") {
        return Ok(endpoint);
    }
    let Some(tls) = tls else {
        return Err(invalid(format!(
            "https copy_port target {url} requires {ENV_TLS_CA} (and typically {ENV_TLS_SERVER_NAME})"
        )));
    };
    endpoint
        .tls_config(tls.load()?)
        .map_err(|e| invalid(format!("tls config for {url}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(
        ca: &str,
        cert: Option<&str>,
        key: Option<&str>,
        name: Option<&str>,
    ) -> ClientTlsOptions {
        ClientTlsOptions {
            ca_path: ca.to_string(),
            cert_path: cert.map(String::from),
            key_path: key.map(String::from),
            server_name: name.map(String::from),
        }
    }

    #[test]
    fn server_options_pairing() {
        assert!(
            ServerTlsOptions::from_paths(None, None, None)
                .unwrap()
                .is_none()
        );
        assert!(
            ServerTlsOptions::from_paths(Some("".into()), Some("".into()), None)
                .unwrap()
                .is_none()
        );
        let some = ServerTlsOptions::from_paths(Some("c".into()), Some("k".into()), None)
            .unwrap()
            .unwrap();
        assert!(some.client_ca_path.is_none());
        assert!(ServerTlsOptions::from_paths(Some("c".into()), None, None).is_err());
        assert!(ServerTlsOptions::from_paths(None, Some("k".into()), None).is_err());
        assert!(ServerTlsOptions::from_paths(None, None, Some("ca".into())).is_err());
    }

    #[test]
    fn server_load_missing_files() {
        let o = ServerTlsOptions::from_paths(
            Some("/nonexistent/tls.crt".into()),
            Some("/nonexistent/tls.key".into()),
            None,
        )
        .unwrap()
        .unwrap();
        assert!(o.load().unwrap_err().to_string().contains("tls cert"));
    }

    #[test]
    #[serial_test::serial(copy_tls_env)]
    fn client_options_from_env() {
        let keys = [ENV_TLS_CA, ENV_TLS_CERT, ENV_TLS_KEY, ENV_TLS_SERVER_NAME];
        let clear = || keys.iter().for_each(|k| unsafe { std::env::remove_var(k) });

        clear();
        assert!(ClientTlsOptions::from_env().unwrap().is_none());

        unsafe { std::env::set_var(ENV_TLS_SERVER_NAME, "x") };
        assert!(ClientTlsOptions::from_env().is_err());

        unsafe { std::env::set_var(ENV_TLS_CA, "/ca.crt") };
        unsafe { std::env::set_var(ENV_TLS_CERT, "/c.crt") };
        assert!(ClientTlsOptions::from_env().is_err());

        unsafe { std::env::set_var(ENV_TLS_KEY, "/c.key") };
        let o = ClientTlsOptions::from_env().unwrap().unwrap();
        assert_eq!(o.ca_path, "/ca.crt");
        assert_eq!(o.server_name.as_deref(), Some("x"));
        clear();
    }

    #[test]
    fn endpoint_scheme_routing() {
        assert!(endpoint_for("http://1.2.3.4:9988", None).is_ok());
        let o = opts("/nonexistent/ca.crt", None, None, Some("name"));
        assert!(endpoint_for("http://1.2.3.4:9988", Some(&o)).is_ok());
        assert!(endpoint_for("https://1.2.3.4:9988", None).is_err());
        assert!(endpoint_for("https://1.2.3.4:9988", Some(&o)).is_err());
    }
}
