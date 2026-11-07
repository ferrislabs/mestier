use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpenseField {
    Label,
    AmountHtCents,
    VatCents,
    DueDate,
    Name,
    Hue,
    Status,
    Category,
}

impl ExpenseField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::AmountHtCents => "amount_ht_cents",
            Self::VatCents => "vat_cents",
            Self::DueDate => "due_date",
            Self::Name => "name",
            Self::Hue => "hue",
            Self::Status => "status",
            Self::Category => "category_id",
        }
    }
}

impl fmt::Display for ExpenseField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ExpenseError {
    #[error("label: must not be empty")]
    LabelRequired,
    #[error("amount_ht_cents: must be greater than zero")]
    AmountNotPositive,
    #[error("vat_cents: must not be negative")]
    VatNegative,
    #[error("due_date: must not be before the expense date")]
    DueBeforeDate,
    #[error("name: must not be empty")]
    NameRequired,
    #[error("hue: must be between 0 and 359")]
    HueOutOfRange,
    #[error("status: only an expense to pay that nobody advanced can be marked paid")]
    NotPayable,
    #[error("status: only an advanced expense not yet reimbursed can be reimbursed")]
    NotReimbursable,
    #[error("category_id: a category used by an expense cannot be deleted")]
    CategoryInUse,
    #[error("name: a category of this name already exists")]
    CategoryNameTaken,
}

impl ExpenseError {
    pub fn field(self) -> ExpenseField {
        match self {
            Self::LabelRequired => ExpenseField::Label,
            Self::AmountNotPositive => ExpenseField::AmountHtCents,
            Self::VatNegative => ExpenseField::VatCents,
            Self::DueBeforeDate => ExpenseField::DueDate,
            Self::NameRequired | Self::CategoryNameTaken => ExpenseField::Name,
            Self::HueOutOfRange => ExpenseField::Hue,
            Self::NotPayable | Self::NotReimbursable => ExpenseField::Status,
            Self::CategoryInUse => ExpenseField::Category,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::NotPayable => "E_EXPENSE_NOT_PAYABLE",
            Self::NotReimbursable => "E_EXPENSE_NOT_REIMBURSABLE",
            Self::CategoryInUse => "E_EXPENSE_CATEGORY_IN_USE",
            Self::CategoryNameTaken => "E_EXPENSE_CATEGORY_NAME_TAKEN",
            Self::LabelRequired
            | Self::AmountNotPositive
            | Self::VatNegative
            | Self::DueBeforeDate
            | Self::NameRequired
            | Self::HueOutOfRange => "E_UNPROCESSABLE_ENTITY",
        }
    }

    pub fn is_conflict(self) -> bool {
        matches!(
            self,
            Self::NotPayable
                | Self::NotReimbursable
                | Self::CategoryInUse
                | Self::CategoryNameTaken
        )
    }
}

pub type ExpenseOutcome<T> = Result<T, ExpenseError>;
