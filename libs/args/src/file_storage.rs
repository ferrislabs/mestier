use common::FileStorageConfig;

const DEFAULT_MAX_UPLOAD_BYTES: u64 = 10 * 1024 * 1024;

#[derive(clap::Args, Debug, Clone)]
pub struct FileStorageArgs {
    #[arg(
        long = "file-storage-bucket",
        env = "FILE_STORAGE_BUCKET",
        default_value = "mestier-files",
        name = "FILE_STORAGE_BUCKET",
        long_help = "S3-compatible bucket used for uploaded files"
    )]
    pub bucket: String,

    #[arg(
        long = "file-storage-endpoint",
        env = "FILE_STORAGE_ENDPOINT",
        default_value = "http://localhost:9000",
        name = "FILE_STORAGE_ENDPOINT",
        long_help = "S3-compatible endpoint URL"
    )]
    pub endpoint: String,

    #[arg(
        long = "file-storage-region",
        env = "FILE_STORAGE_REGION",
        default_value = "us-east-1",
        name = "FILE_STORAGE_REGION",
        long_help = "S3-compatible region"
    )]
    pub region: String,

    #[arg(
        long = "file-storage-access-key-id",
        env = "FILE_STORAGE_ACCESS_KEY_ID",
        default_value = "rustfsadmin",
        name = "FILE_STORAGE_ACCESS_KEY_ID",
        long_help = "S3-compatible access key id"
    )]
    pub access_key_id: String,

    #[arg(
        long = "file-storage-secret-access-key",
        env = "FILE_STORAGE_SECRET_ACCESS_KEY",
        default_value = "rustfsadmin",
        name = "FILE_STORAGE_SECRET_ACCESS_KEY",
        long_help = "S3-compatible secret access key"
    )]
    pub secret_access_key: String,

    #[arg(
        long = "file-storage-force-path-style",
        env = "FILE_STORAGE_FORCE_PATH_STYLE",
        default_value_t = true,
        name = "FILE_STORAGE_FORCE_PATH_STYLE",
        long_help = "Use S3 path-style addressing; required by local RustFS"
    )]
    pub force_path_style: bool,

    // `ArgAction::Set`, so the option takes a value. As a bare flag it could
    // only ever be turned *on*, and it already defaults to on — there was no way
    // to say no from the command line, which made a reachable object store a
    // prerequisite for anything that builds the service, the end-to-end suites
    // included. The environment variable keeps working unchanged.
    #[arg(
        long = "file-storage-auto-create-bucket",
        env = "FILE_STORAGE_AUTO_CREATE_BUCKET",
        action = clap::ArgAction::Set,
        default_value_t = true,
        name = "FILE_STORAGE_AUTO_CREATE_BUCKET",
        long_help = "Create the configured bucket at startup when missing"
    )]
    pub auto_create_bucket: bool,

    #[arg(
        long = "file-storage-key-prefix",
        env = "FILE_STORAGE_KEY_PREFIX",
        default_value = "uploads",
        name = "FILE_STORAGE_KEY_PREFIX",
        long_help = "Prefix applied to generated object keys"
    )]
    pub key_prefix: String,

    #[arg(
        long = "file-storage-max-upload-bytes",
        env = "FILE_STORAGE_MAX_UPLOAD_BYTES",
        default_value_t = DEFAULT_MAX_UPLOAD_BYTES,
        name = "FILE_STORAGE_MAX_UPLOAD_BYTES",
        long_help = "Maximum accepted upload size in bytes"
    )]
    pub max_upload_bytes: u64,
}

impl Default for FileStorageArgs {
    fn default() -> Self {
        Self {
            bucket: "mestier-files".to_owned(),
            endpoint: "http://localhost:9000".to_owned(),
            region: "us-east-1".to_owned(),
            access_key_id: "rustfsadmin".to_owned(),
            secret_access_key: "rustfsadmin".to_owned(),
            force_path_style: true,
            auto_create_bucket: true,
            key_prefix: "uploads".to_owned(),
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
        }
    }
}

impl From<FileStorageArgs> for FileStorageConfig {
    fn from(value: FileStorageArgs) -> Self {
        Self {
            bucket: value.bucket,
            endpoint: value.endpoint,
            region: value.region,
            access_key_id: value.access_key_id,
            secret_access_key: value.secret_access_key,
            force_path_style: value.force_path_style,
            auto_create_bucket: value.auto_create_bucket,
            key_prefix: value.key_prefix,
            max_upload_bytes: value.max_upload_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_local_rustfs() {
        let args = FileStorageArgs::default();

        assert_eq!(args.bucket, "mestier-files");
        assert_eq!(args.endpoint, "http://localhost:9000");
        assert_eq!(args.region, "us-east-1");
        assert_eq!(args.access_key_id, "rustfsadmin");
        assert_eq!(args.secret_access_key, "rustfsadmin");
        assert!(args.force_path_style);
        assert!(args.auto_create_bucket);
        assert_eq!(args.key_prefix, "uploads");
        assert_eq!(args.max_upload_bytes, DEFAULT_MAX_UPLOAD_BYTES);
    }
}
