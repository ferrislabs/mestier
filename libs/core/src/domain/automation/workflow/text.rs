pub const MAX_WORKFLOW_NAME_LENGTH: usize = 120;
pub const MAX_WORKFLOW_DESCRIPTION_LENGTH: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowName(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WorkflowNameError {
    #[error("a workflow name cannot be empty")]
    Empty,
    #[error("a workflow name holds at most {MAX_WORKFLOW_NAME_LENGTH} characters")]
    TooLong,
}

impl WorkflowName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for WorkflowName {
    type Error = WorkflowNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(WorkflowNameError::Empty);
        }
        if trimmed.chars().count() > MAX_WORKFLOW_NAME_LENGTH {
            return Err(WorkflowNameError::TooLong);
        }
        if trimmed.len() == raw.len() {
            return Ok(Self(raw));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDescription(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WorkflowDescriptionError {
    #[error("a workflow description holds at most {MAX_WORKFLOW_DESCRIPTION_LENGTH} characters")]
    TooLong,
}

impl WorkflowDescription {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for WorkflowDescription {
    type Error = WorkflowDescriptionError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.chars().count() > MAX_WORKFLOW_DESCRIPTION_LENGTH {
            return Err(WorkflowDescriptionError::TooLong);
        }
        Ok(Self(raw))
    }
}
