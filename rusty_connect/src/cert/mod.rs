use std::io;

use base64::Engine;

pub mod certgen;
pub mod no_veifier;

pub type CertPair = (Vec<u8>, Vec<u8>);

fn der_to_pem_cert(der_bytes: &[u8]) -> io::Result<Vec<u8>> {
    const PEM_HEADER: &str = "-----BEGIN CERTIFICATE-----\n";
    const PEM_FOOTER: &str = "\n-----END CERTIFICATE-----\n";
    let mut pem_bytes = Vec::new();
    pem_bytes.extend_from_slice(PEM_HEADER.as_bytes());
    pem_bytes.extend_from_slice(
        &base64::prelude::BASE64_STANDARD
            .encode(der_bytes)
            .into_bytes(),
    );
    pem_bytes.extend_from_slice(PEM_FOOTER.as_bytes());

    Ok(pem_bytes)
}

fn der_to_pem_key(der_bytes: &[u8]) -> io::Result<Vec<u8>> {
    const PEM_HEADER: &str = "-----BEGIN PRIVATE KEY-----\n";
    const PEM_FOOTER: &str = "\n-----END PRIVATE KEY-----\n";
    let mut pem_bytes = Vec::new();
    pem_bytes.extend_from_slice(PEM_HEADER.as_bytes());
    pem_bytes.extend_from_slice(
        &base64::prelude::BASE64_STANDARD
            .encode(der_bytes)
            .into_bytes(),
    );
    pem_bytes.extend_from_slice(PEM_FOOTER.as_bytes());

    Ok(pem_bytes)
}
