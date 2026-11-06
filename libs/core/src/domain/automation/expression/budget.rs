use std::io::{self, Write};

use serde_json::Value;

use super::error::ExpressionError;

pub const MAX_RESOLVED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct EvalBudget {
    remaining: usize,
    limit: usize,
}

impl EvalBudget {
    pub fn new(limit: usize) -> Self {
        Self {
            remaining: limit,
            limit,
        }
    }

    pub fn charge(&mut self, bytes: usize) -> Result<(), ExpressionError> {
        if bytes > self.remaining {
            self.remaining = 0;
            return Err(ExpressionError::TooLarge { limit: self.limit });
        }
        self.remaining -= bytes;
        Ok(())
    }
}

impl Default for EvalBudget {
    fn default() -> Self {
        Self::new(MAX_RESOLVED_BYTES)
    }
}

struct Counter(usize);

impl Write for Counter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn json_size(value: &Value) -> usize {
    let mut counter = Counter(0);
    match serde_json::to_writer(&mut counter, value) {
        Ok(()) => counter.0,
        Err(_) => usize::MAX,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_value_weighs_its_serialized_json_form() {
        let value = json!({ "a": "é\n", "b": [1, 2.5, null, true] });

        assert_eq!(
            json_size(&value),
            serde_json::to_string(&value).unwrap().len()
        );
        assert_eq!(json_size(&json!("x")), 3);
    }

    #[test]
    fn a_budget_refuses_the_charge_that_crosses_it_and_every_one_after() {
        let mut budget = EvalBudget::new(10);

        assert!(budget.charge(6).is_ok());
        assert_eq!(
            budget.charge(5).unwrap_err(),
            ExpressionError::TooLarge { limit: 10 }
        );
        assert_eq!(
            budget.charge(1).unwrap_err(),
            ExpressionError::TooLarge { limit: 10 }
        );
    }

    #[test]
    fn a_charge_of_exactly_what_is_left_is_accepted() {
        let mut budget = EvalBudget::new(10);

        assert!(budget.charge(10).is_ok());
        assert!(budget.charge(0).is_ok());
        assert!(budget.charge(1).is_err());
    }
}
