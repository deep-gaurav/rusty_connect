use std::path::Path;

use rcgen::{CertificateParams, DistinguishedName};

pub async fn generate_cert(
    device_id: &str,
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let cert_exist = futures::join!(
        tokio::fs::try_exists(&cert_path),
        tokio::fs::try_exists(&key_path)
    );
    if let (Ok(true), Ok(true)) = cert_exist {
        let cert = tokio::fs::read(cert_path).await?;
        let key = tokio::fs::read(key_path).await?;
        Ok((cert, key))
    } else {
        let mut params = CertificateParams::default();
        let mut distin_name = DistinguishedName::new();
        distin_name.push(rcgen::DnType::CommonName, device_id);
        distin_name.push(rcgen::DnType::OrganizationName, "Deep"); //TODO: dont hard code me?
        distin_name.push(rcgen::DnType::OrganizationalUnitName, "RustyConnect"); //TODO: dont hard code RustyConnect?
        params.distinguished_name = distin_name;
        let cert = rcgen::Certificate::from_params(params).unwrap();
        tokio::fs::write(&cert_path, &cert.serialize_der()?).await?;
        tokio::fs::write(&key_path, &cert.serialize_private_key_der()).await?;

        Ok((
            cert.serialize_der().unwrap(),
            cert.serialize_private_key_der(),
        ))
    }
}
