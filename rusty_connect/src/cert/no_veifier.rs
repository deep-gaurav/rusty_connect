// use tokio_rustls::rustls::{
//     client::danger::ServerCertVerifier, server::danger::ClientCertVerifier,
// };

// #[derive(Debug)]
// pub struct NoVerifier;

// impl ClientCertVerifier for NoVerifier {
//     fn root_hint_subjects(&self) -> &[tokio_rustls::rustls::DistinguishedName] {
//         &[]
//     }

//     fn verify_client_cert(
//         &self,
//         _end_entity: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
//         _intermediates: &[tokio_rustls::rustls::pki_types::CertificateDer<'_>],
//         _now: tokio_rustls::rustls::pki_types::UnixTime,
//     ) -> Result<tokio_rustls::rustls::server::danger::ClientCertVerified, tokio_rustls::rustls::Error>
//     {
//         Ok(tokio_rustls::rustls::server::danger::ClientCertVerified::assertion())
//     }

//     fn verify_tls12_signature(
//         &self,
//         _message: &[u8],
//         _cert: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
//         _dss: &tokio_rustls::rustls::DigitallySignedStruct,
//     ) -> Result<
//         tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
//         tokio_rustls::rustls::Error,
//     > {
//         Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
//     }

//     fn verify_tls13_signature(
//         &self,
//         _message: &[u8],
//         _cert: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
//         _dss: &tokio_rustls::rustls::DigitallySignedStruct,
//     ) -> Result<
//         tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
//         tokio_rustls::rustls::Error,
//     > {
//         Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
//     }

//     fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
//         vec![
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA256,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA1,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_SHA1_Legacy,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA256,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA384,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA512,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA256,
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA384,
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA512,
//             tokio_rustls::rustls::SignatureScheme::ED25519,
//             tokio_rustls::rustls::SignatureScheme::ED448,
//         ]
//     }
// }

// impl ServerCertVerifier for NoVerifier {
//     fn verify_server_cert(
//         &self,
//         _end_entity: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
//         _intermediates: &[tokio_rustls::rustls::pki_types::CertificateDer<'_>],
//         _server_name: &tokio_rustls::rustls::pki_types::ServerName<'_>,
//         _ocsp_response: &[u8],
//         _now: tokio_rustls::rustls::pki_types::UnixTime,
//     ) -> Result<tokio_rustls::rustls::client::danger::ServerCertVerified, tokio_rustls::rustls::Error>
//     {
//         Ok(tokio_rustls::rustls::client::danger::ServerCertVerified::assertion())
//     }

//     fn verify_tls12_signature(
//         &self,
//         _message: &[u8],
//         _cert: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
//         _dss: &tokio_rustls::rustls::DigitallySignedStruct,
//     ) -> Result<
//         tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
//         tokio_rustls::rustls::Error,
//     > {
//         Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
//     }

//     fn verify_tls13_signature(
//         &self,
//         _message: &[u8],
//         _cert: &tokio_rustls::rustls::pki_types::CertificateDer<'_>,
//         _dss: &tokio_rustls::rustls::DigitallySignedStruct,
//     ) -> Result<
//         tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
//         tokio_rustls::rustls::Error,
//     > {
//         Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
//     }

//     fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
//         vec![
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA256,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA1,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_SHA1_Legacy,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA256,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA384,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
//             tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA512,
//             tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA256,
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA384,
//             tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA512,
//             tokio_rustls::rustls::SignatureScheme::ED25519,
//             tokio_rustls::rustls::SignatureScheme::ED448,
//         ]
//     }
// }
