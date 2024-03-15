use std::path::{Path, PathBuf};

use rand::rngs::OsRng;
use rcgen::{CertificateParams, DistinguishedName};
use rsa::RsaPrivateKey;
use tracing::info;

use super::{der_to_pem_cert, der_to_pem_key, CertPair};
use rsa::pkcs8::EncodePrivateKey;

pub async fn generate_cert(
    device_id: &str,
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> anyhow::Result<CertPair> {
    let cert_exist = futures::join!(
        tokio::fs::try_exists(&cert_path),
        tokio::fs::try_exists(&key_path)
    );
    let (cert, key) = if let (Ok(true), Ok(true)) = cert_exist {
        let cert = tokio::fs::read(&cert_path).await?;
        let key = tokio::fs::read(&key_path).await?;
        (cert, key)
    } else {
        let mut params = CertificateParams::default();
        params.alg = &rcgen::PKCS_RSA_SHA256;

        let mut rng = OsRng;
        let bits = 2048;
        info!("Generating RSA private key");
        let private_key = RsaPrivateKey::new(&mut rng, bits)?;
        info!("Serializing RSA private key");

        let private_key_der = private_key.to_pkcs8_der()?;

        info!("Creating key pair");

        let key_pair = rcgen::KeyPair::try_from(private_key_der.as_bytes())?;
        params.key_pair = Some(key_pair);

        let mut distin_name = DistinguishedName::new();
        distin_name.push(rcgen::DnType::CommonName, device_id);
        distin_name.push(rcgen::DnType::OrganizationName, "Deep"); //TODO: dont hard code me?
        distin_name.push(rcgen::DnType::OrganizationalUnitName, "RustyConnect"); //TODO: dont hard code RustyConnect?
        params.distinguished_name = distin_name;

        info!("Generating Certificate");

        let cert = rcgen::Certificate::from_params(params)?;

        info!("Writing Certificate");

        tokio::fs::write(&cert_path, &cert.serialize_pem()?).await?;
        tokio::fs::write(&key_path, &cert.serialize_private_key_pem()).await?;

        (
            cert.serialize_pem().unwrap().as_bytes().to_vec(),
            cert.serialize_private_key_pem().as_bytes().to_vec(),
        )
    };
    let (pem_cert, pem_key) = (cert, key);
    // let pem_cert = der_to_pem_cert(&cert)?;
    // let pem_key = der_to_pem_key(&key)?;

    // tokio::fs::write(
    //     &format!("{}pem", PathBuf::new().join(cert_path).to_str().unwrap()),
    //     &pem_cert,
    // )
    // .await?;

    // tokio::fs::write(
    //     &format!("{}pem", PathBuf::new().join(key_path).to_str().unwrap()),
    //     &pem_key,
    // )
    // .await?;
    Ok((pem_cert, pem_key))
}
