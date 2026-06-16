#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadFileCommand {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}
