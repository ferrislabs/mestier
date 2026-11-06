use common::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CollectionError {
    #[error("the collection does not exist in this organization")]
    NotFound,
    #[error("the collection model holds {size} bytes, over the limit of {limit} bytes")]
    Oversized { size: usize, limit: usize },
    #[error("invalid collection: {0}")]
    InvalidModel(String),
}

impl From<CollectionError> for CoreError {
    fn from(error: CollectionError) -> Self {
        match error {
            CollectionError::NotFound => Self::NotFound,
            other => Self::Conflict(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_stays_not_found_for_the_core() {
        assert_eq!(
            CoreError::from(CollectionError::NotFound).to_string(),
            CoreError::NotFound.to_string()
        );
    }

    #[test]
    fn an_oversized_model_is_a_conflict_that_names_both_sizes() {
        let error = CoreError::from(CollectionError::Oversized { size: 3, limit: 2 });

        let CoreError::Conflict(message) = error else {
            panic!("expected a conflict, got {error:?}");
        };
        assert!(message.contains('3') && message.contains('2'), "{message}");
    }

    #[test]
    fn an_invalid_model_is_a_conflict_that_carries_the_reason() {
        let error = CoreError::from(CollectionError::InvalidModel("no operation".to_owned()));

        let CoreError::Conflict(message) = error else {
            panic!("expected a conflict, got {error:?}");
        };
        assert!(message.contains("no operation"), "{message}");
    }
}
