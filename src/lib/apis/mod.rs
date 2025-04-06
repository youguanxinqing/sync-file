pub mod download;
pub mod ping;
pub mod upload;

pub mod urls {
    #[macro_export]
    macro_rules! __PING_URL_V1 {
        ($protocol:expr, $addr:expr) => {
            format!("{}://{}/ping", $protocol, $addr)
        };
    }
    pub use __PING_URL_V1 as PING_URL_V1;

    #[macro_export]
    macro_rules! __UPLOAD_URL_V1 {
        ($protocol:expr, $addr:expr) => {
            format!("{}://{}/upload", $protocol, $addr)
        };
    }
    pub use __UPLOAD_URL_V1 as UPLOAD_URL_V1;

    #[macro_export]
    macro_rules! __DOWNLOAD_URL_V1 {
        ($protocol:expr, $addr:expr) => {
            format!("{}://{}/download", $protocol, $addr)
        };
    }
    pub use __DOWNLOAD_URL_V1 as DOWNLOAD_URL_V1;
}
