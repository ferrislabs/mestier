#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadFileCommand {
    pub mime_type: String,
    pub bytes: Vec<u8>,
    pub folder: Option<String>,
}
