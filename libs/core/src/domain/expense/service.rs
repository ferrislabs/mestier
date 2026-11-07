use authz::{Authorizer, Resource, Subject};
use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};

use super::{
    Expense, ExpenseCategory, ExpenseCategoryId, ExpenseError, ExpenseId, ExpenseOutcome,
    ExpenseStatus,
    commands::{
        CreateExpenseCategoryCommand, CreateExpenseCommand, MarkExpensePaidCommand,
        UpdateExpenseCategoryCommand, UpdateExpenseCommand,
    },
    ports::{ExpenseCategoryRepository, ExpenseRepository},
};
use crate::{
    CustomerId, MemberId, OrganizationId, PaymentMethod, SupplierId,
    application::policy,
    domain::{
        customer::ports::CustomerRepository, member::ports::MemberRepository,
        role::ports::RoleRepository, supplier_invoice::ports::SupplierRepository,
    },
};

pub(super) const DEFAULT_CATEGORIES: [(&str, i32); 9] = [
    ("Matériaux", 60),
    ("Sous-traitance", 200),
    ("Véhicules et carburant", 25),
    ("Outillage et consommables", 110),
    ("Assurances", 280),
    ("Loyers et charges", 330),
    ("Logiciels et télécoms", 230),
    ("Déplacements et repas", 160),
    ("Autres", 0),
];

struct ValidExpenseFields {
    label: String,
    reference: Option<String>,
    note: Option<String>,
    proof_file_key: Option<String>,
}

struct RawExpenseFields {
    date: NaiveDate,
    label: String,
    reference: Option<String>,
    amount_ht_cents: i32,
    vat_cents: i32,
    due_date: Option<NaiveDate>,
    proof_file_key: Option<String>,
    note: Option<String>,
}

fn validate_fields(raw: RawExpenseFields) -> ExpenseOutcome<ValidExpenseFields> {
    let label = raw.label.trim().to_owned();
    if label.is_empty() {
        return Err(ExpenseError::LabelRequired);
    }
    if raw.amount_ht_cents <= 0 {
        return Err(ExpenseError::AmountNotPositive);
    }
    if raw.vat_cents < 0 {
        return Err(ExpenseError::VatNegative);
    }
    if raw.due_date.is_some_and(|due| due < raw.date) {
        return Err(ExpenseError::DueBeforeDate);
    }
    Ok(ValidExpenseFields {
        label,
        reference: super::clean_text(raw.reference),
        note: super::clean_text(raw.note),
        proof_file_key: super::clean_text(raw.proof_file_key),
    })
}

async fn authorize_write<M, Ro, A>(
    members: &mut M,
    roles: &mut Ro,
    authz: &A,
    actor: Subject,
    organization_id: OrganizationId,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "invoice.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

pub struct ExpenseService<E, C, S, Cu, M, Ro, A>
where
    E: ExpenseRepository,
    C: ExpenseCategoryRepository,
    S: SupplierRepository,
    Cu: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    expenses: E,
    categories: C,
    suppliers: S,
    customers: Cu,
    member_repository: M,
    role_repository: Ro,
    authz: A,
}

impl<E, C, S, Cu, M, Ro, A> ExpenseService<E, C, S, Cu, M, Ro, A>
where
    E: ExpenseRepository,
    C: ExpenseCategoryRepository,
    S: SupplierRepository,
    Cu: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(
        expenses: E,
        categories: C,
        suppliers: S,
        customers: Cu,
        member_repository: M,
        role_repository: Ro,
        authz: A,
    ) -> Self {
        Self {
            expenses,
            categories,
            suppliers,
            customers,
            member_repository,
            role_repository,
            authz,
        }
    }

    async fn authorize(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        authorize_write(
            &mut self.member_repository,
            &mut self.role_repository,
            &self.authz,
            actor,
            organization_id,
        )
        .await
    }

    async fn require_references(
        &mut self,
        organization_id: OrganizationId,
        category_id: ExpenseCategoryId,
        supplier_id: Option<SupplierId>,
        customer_id: Option<CustomerId>,
        advanced_by: Option<MemberId>,
    ) -> Result<(), CoreError> {
        self.categories
            .find_by_id(category_id)
            .await?
            .filter(|category| category.organization_id == organization_id)
            .ok_or(CoreError::NotFound)?;
        if let Some(supplier_id) = supplier_id {
            self.suppliers
                .find_by_id(supplier_id)
                .await?
                .filter(|supplier| supplier.organization_id == organization_id)
                .ok_or(CoreError::NotFound)?;
        }
        if let Some(customer_id) = customer_id {
            self.customers
                .find_by_id(customer_id)
                .await?
                .filter(|customer| customer.organization_id == organization_id)
                .ok_or(CoreError::NotFound)?;
        }
        if let Some(member_id) = advanced_by {
            self.member_repository
                .find_by_id(member_id)
                .await?
                .filter(|member| member.organization_id == organization_id)
                .ok_or(CoreError::NotFound)?;
        }
        Ok(())
    }

    pub async fn create_expense(
        &mut self,
        command: CreateExpenseCommand,
        now: DateTime<Utc>,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let fields = match validate_fields(RawExpenseFields {
            date: command.date,
            label: command.label,
            reference: command.reference,
            amount_ht_cents: command.amount_ht_cents,
            vat_cents: command.vat_cents,
            due_date: command.due_date,
            proof_file_key: command.proof_file_key,
            note: command.note,
        }) {
            Ok(fields) => fields,
            Err(refusal) => return Ok(Err(refusal)),
        };

        self.authorize(command.actor, command.organization_id)
            .await?;
        self.require_references(
            command.organization_id,
            command.category_id,
            command.supplier_id,
            command.customer_id,
            command.advanced_by_member_id,
        )
        .await?;

        let advanced = command.advanced_by_member_id.is_some();
        let expense = Expense {
            id: ExpenseId(generate_uuid_v7()),
            organization_id: command.organization_id,
            date: command.date,
            label: fields.label,
            supplier_id: command.supplier_id,
            category_id: command.category_id,
            reference: fields.reference,
            amount_ht_cents: command.amount_ht_cents,
            vat_cents: command.vat_cents,
            due_date: if advanced { None } else { command.due_date },
            status: if advanced {
                ExpenseStatus::Paid
            } else {
                ExpenseStatus::ToPay
            },
            paid_on: advanced.then_some(command.date),
            method: advanced.then_some(PaymentMethod::Card),
            advanced_by_member_id: command.advanced_by_member_id,
            reimbursed_on: None,
            customer_id: command.customer_id,
            proof_file_key: fields.proof_file_key,
            note: fields.note,
            created_at: now,
            updated_at: now,
            supplier_name: None,
            customer_name: None,
        };

        self.expenses.insert(&expense).await.map(Ok)
    }

    pub async fn get_expense(&mut self, id: ExpenseId) -> Result<Expense, CoreError> {
        self.expenses
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_expenses(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Expense>, CoreError> {
        self.expenses.list_by_organization(organization_id).await
    }

    pub async fn update_expense(
        &mut self,
        command: UpdateExpenseCommand,
        now: DateTime<Utc>,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let mut expense = self.get_expense(command.id).await?;
        let fields = match validate_fields(RawExpenseFields {
            date: command.date,
            label: command.label,
            reference: command.reference,
            amount_ht_cents: command.amount_ht_cents,
            vat_cents: command.vat_cents,
            due_date: command.due_date,
            proof_file_key: command.proof_file_key,
            note: command.note,
        }) {
            Ok(fields) => fields,
            Err(refusal) => return Ok(Err(refusal)),
        };

        self.authorize(command.actor, expense.organization_id)
            .await?;
        self.require_references(
            expense.organization_id,
            command.category_id,
            command.supplier_id,
            command.customer_id,
            None,
        )
        .await?;

        let advanced = expense.advanced_by_member_id.is_some();
        expense.date = command.date;
        expense.label = fields.label;
        expense.supplier_id = command.supplier_id;
        expense.category_id = command.category_id;
        expense.reference = fields.reference;
        expense.amount_ht_cents = command.amount_ht_cents;
        expense.vat_cents = command.vat_cents;
        expense.due_date = if advanced { None } else { command.due_date };
        expense.customer_id = command.customer_id;
        expense.proof_file_key = fields.proof_file_key;
        expense.note = fields.note;
        expense.updated_at = now;

        self.expenses.update(&expense).await.map(Ok)
    }

    pub async fn mark_paid(
        &mut self,
        command: MarkExpensePaidCommand,
        now: DateTime<Utc>,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let expense = self.get_expense(command.id).await?;
        self.authorize(command.actor, expense.organization_id)
            .await?;
        if expense.status != ExpenseStatus::ToPay || expense.advanced_by_member_id.is_some() {
            return Ok(Err(ExpenseError::NotPayable));
        }
        Ok(self
            .expenses
            .mark_paid(command.id, command.paid_on, command.method, now)
            .await?
            .ok_or(ExpenseError::NotPayable))
    }

    pub async fn reimburse(
        &mut self,
        id: ExpenseId,
        actor: Subject,
        today: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let expense = self.get_expense(id).await?;
        self.authorize(actor, expense.organization_id).await?;
        if expense.advanced_by_member_id.is_none() || expense.reimbursed_on.is_some() {
            return Ok(Err(ExpenseError::NotReimbursable));
        }
        Ok(self
            .expenses
            .reimburse(id, today, now)
            .await?
            .ok_or(ExpenseError::NotReimbursable))
    }

    pub async fn delete_expense(&mut self, id: ExpenseId, actor: Subject) -> Result<(), CoreError> {
        let expense = self.get_expense(id).await?;
        self.authorize(actor, expense.organization_id).await?;
        self.expenses.delete(id).await
    }
}

pub struct ExpenseCategoryService<C, M, Ro, A>
where
    C: ExpenseCategoryRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    categories: C,
    member_repository: M,
    role_repository: Ro,
    authz: A,
}

fn validate_category(name: &str, hue: i64) -> ExpenseOutcome<(String, i32)> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(ExpenseError::NameRequired);
    }
    let hue = i32::try_from(hue)
        .ok()
        .filter(|hue| (0..360).contains(hue))
        .ok_or(ExpenseError::HueOutOfRange)?;
    Ok((name, hue))
}

impl<C, M, Ro, A> ExpenseCategoryService<C, M, Ro, A>
where
    C: ExpenseCategoryRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(categories: C, member_repository: M, role_repository: Ro, authz: A) -> Self {
        Self {
            categories,
            member_repository,
            role_repository,
            authz,
        }
    }

    async fn authorize(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        authorize_write(
            &mut self.member_repository,
            &mut self.role_repository,
            &self.authz,
            actor,
            organization_id,
        )
        .await
    }

    async fn name_taken(
        &mut self,
        organization_id: OrganizationId,
        name: &str,
        except: Option<ExpenseCategoryId>,
    ) -> Result<bool, CoreError> {
        let wanted = name.to_lowercase();
        Ok(self
            .categories
            .list_by_organization(organization_id)
            .await?
            .iter()
            .any(|category| Some(category.id) != except && category.name.to_lowercase() == wanted))
    }

    pub async fn list_categories(
        &mut self,
        organization_id: OrganizationId,
        now: DateTime<Utc>,
    ) -> Result<Vec<ExpenseCategory>, CoreError> {
        let existing = self
            .categories
            .list_by_organization(organization_id)
            .await?;
        if !existing.is_empty() {
            return Ok(existing);
        }
        for (name, hue) in DEFAULT_CATEGORIES {
            self.categories
                .insert(&ExpenseCategory {
                    id: ExpenseCategoryId(generate_uuid_v7()),
                    organization_id,
                    name: name.to_owned(),
                    hue,
                    created_at: now,
                    updated_at: now,
                })
                .await?;
        }
        self.categories.list_by_organization(organization_id).await
    }

    pub async fn get_category(
        &mut self,
        id: ExpenseCategoryId,
    ) -> Result<ExpenseCategory, CoreError> {
        self.categories
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn create_category(
        &mut self,
        command: CreateExpenseCategoryCommand,
        now: DateTime<Utc>,
    ) -> Result<ExpenseOutcome<ExpenseCategory>, CoreError> {
        let (name, hue) = match validate_category(&command.name, command.hue) {
            Ok(valid) => valid,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.authorize(command.actor, command.organization_id)
            .await?;
        if self
            .name_taken(command.organization_id, &name, None)
            .await?
        {
            return Ok(Err(ExpenseError::CategoryNameTaken));
        }
        self.categories
            .insert(&ExpenseCategory {
                id: ExpenseCategoryId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name,
                hue,
                created_at: now,
                updated_at: now,
            })
            .await
            .map(Ok)
    }

    pub async fn update_category(
        &mut self,
        command: UpdateExpenseCategoryCommand,
        now: DateTime<Utc>,
    ) -> Result<ExpenseOutcome<ExpenseCategory>, CoreError> {
        let mut category = self.get_category(command.id).await?;
        let (name, hue) = match validate_category(&command.name, command.hue) {
            Ok(valid) => valid,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.authorize(command.actor, category.organization_id)
            .await?;
        if self
            .name_taken(category.organization_id, &name, Some(category.id))
            .await?
        {
            return Ok(Err(ExpenseError::CategoryNameTaken));
        }
        category.name = name;
        category.hue = hue;
        category.updated_at = now;
        self.categories.update(&category).await.map(Ok)
    }

    pub async fn delete_category(
        &mut self,
        id: ExpenseCategoryId,
        actor: Subject,
    ) -> Result<ExpenseOutcome<()>, CoreError> {
        let category = self.get_category(id).await?;
        self.authorize(actor, category.organization_id).await?;
        if self.categories.is_used(id).await? {
            return Ok(Err(ExpenseError::CategoryInUse));
        }
        self.categories.delete(id).await.map(Ok)
    }
}

#[cfg(test)]
mod tests;
