use std::sync::{Arc, Mutex};

use authz::{Decision, MockAuthorizer, Subject};
use chrono::{DateTime, NaiveDate, Utc};
use common::generate_uuid_v7;
use mockall::predicate::eq;

use crate::{
    Customer, CustomerId, CustomerProfile, CustomerStatus, Expense, ExpenseCategory,
    ExpenseCategoryId, ExpenseDisplayState, ExpenseError, ExpenseId, ExpenseStatus, Member,
    MemberId, MockCustomerRepository, MockExpenseCategoryRepository, MockExpenseRepository,
    MockMemberRepository, MockRoleRepository, MockSupplierRepository, OrganizationId,
    PaymentMethod, Supplier, SupplierId, SupplierProfile,
    domain::expense::{
        ExpenseCategoryService, ExpenseService,
        commands::{
            CreateExpenseCategoryCommand, CreateExpenseCommand, MarkExpensePaidCommand,
            UpdateExpenseCommand,
        },
        service::DEFAULT_CATEGORIES,
    },
};

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
}

fn now() -> DateTime<Utc> {
    date(10).and_hms_opt(12, 0, 0).unwrap().and_utc()
}

fn allow_all() -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    authz
}

fn category(organization_id: OrganizationId) -> ExpenseCategory {
    ExpenseCategory {
        id: ExpenseCategoryId(generate_uuid_v7()),
        organization_id,
        name: "Matériaux".to_owned(),
        hue: 60,
        created_at: now(),
        updated_at: now(),
    }
}

fn expense(organization_id: OrganizationId) -> Expense {
    Expense {
        id: ExpenseId(generate_uuid_v7()),
        organization_id,
        date: date(1),
        label: "Quincaillerie".to_owned(),
        supplier_id: None,
        category_id: ExpenseCategoryId(generate_uuid_v7()),
        reference: None,
        amount_ht_cents: 10_000,
        vat_cents: 2_000,
        due_date: Some(date(20)),
        status: ExpenseStatus::ToPay,
        paid_on: None,
        method: None,
        advanced_by_member_id: None,
        reimbursed_on: None,
        customer_id: None,
        proof_file_key: None,
        note: None,
        created_at: now(),
        updated_at: now(),
        supplier_name: None,
        customer_name: None,
    }
}

fn command(
    organization_id: OrganizationId,
    category_id: ExpenseCategoryId,
) -> CreateExpenseCommand {
    CreateExpenseCommand {
        actor: Subject::system(),
        organization_id,
        date: date(1),
        label: " Quincaillerie ".to_owned(),
        supplier_id: None,
        category_id,
        reference: Some("  ".to_owned()),
        amount_ht_cents: 10_000,
        vat_cents: 2_000,
        due_date: Some(date(20)),
        advanced_by_member_id: None,
        customer_id: None,
        proof_file_key: None,
        note: None,
    }
}

type Service = ExpenseService<
    MockExpenseRepository,
    MockExpenseCategoryRepository,
    MockSupplierRepository,
    MockCustomerRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

fn make_service(
    expenses: MockExpenseRepository,
    categories: MockExpenseCategoryRepository,
    suppliers: MockSupplierRepository,
    customers: MockCustomerRepository,
    members: MockMemberRepository,
) -> Service {
    ExpenseService::new(
        expenses,
        categories,
        suppliers,
        customers,
        members,
        MockRoleRepository::new(),
        allow_all(),
    )
}

fn categories_with(category: ExpenseCategory) -> MockExpenseCategoryRepository {
    let mut categories = MockExpenseCategoryRepository::new();
    categories.expect_find_by_id().returning(move |_| {
        let category = category.clone();
        Box::pin(async move { Ok(Some(category)) })
    });
    categories
}

fn inserting() -> MockExpenseRepository {
    let mut expenses = MockExpenseRepository::new();
    expenses.expect_insert().returning(|expense| {
        let expense = expense.clone();
        Box::pin(async move { Ok(expense) })
    });
    expenses
}

fn bare(expenses: MockExpenseRepository, categories: MockExpenseCategoryRepository) -> Service {
    make_service(
        expenses,
        categories,
        MockSupplierRepository::new(),
        MockCustomerRepository::new(),
        MockMemberRepository::new(),
    )
}

#[tokio::test]
async fn an_expense_needs_a_label_an_amount_a_vat_and_a_due_date_in_order() {
    let org = OrganizationId(generate_uuid_v7());
    let category = category(org);
    let cases = [
        (
            CreateExpenseCommand {
                label: "   ".to_owned(),
                ..command(org, category.id)
            },
            ExpenseError::LabelRequired,
        ),
        (
            CreateExpenseCommand {
                amount_ht_cents: 0,
                ..command(org, category.id)
            },
            ExpenseError::AmountNotPositive,
        ),
        (
            CreateExpenseCommand {
                vat_cents: -1,
                ..command(org, category.id)
            },
            ExpenseError::VatNegative,
        ),
        (
            CreateExpenseCommand {
                due_date: Some(date(1).pred_opt().unwrap()),
                ..command(org, category.id)
            },
            ExpenseError::DueBeforeDate,
        ),
    ];

    for (case, expected) in cases {
        let mut service = bare(
            MockExpenseRepository::new(),
            categories_with(category.clone()),
        );

        let outcome = service.create_expense(case, now()).await.unwrap();

        assert_eq!(outcome.unwrap_err(), expected);
    }
}

#[tokio::test]
async fn a_zero_vat_and_a_due_date_equal_to_the_date_are_accepted() {
    let org = OrganizationId(generate_uuid_v7());
    let category = category(org);
    let mut service = bare(inserting(), categories_with(category.clone()));

    let created = service
        .create_expense(
            CreateExpenseCommand {
                vat_cents: 0,
                due_date: Some(date(1)),
                ..command(org, category.id)
            },
            now(),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(created.label, "Quincaillerie");
    assert_eq!(created.reference, None);
    assert_eq!(created.status, ExpenseStatus::ToPay);
    assert_eq!(created.total_cents(), 10_000);
}

#[tokio::test]
async fn an_unknown_category_or_one_from_another_organization_is_not_found() {
    let org = OrganizationId(generate_uuid_v7());
    let foreign = category(OrganizationId(generate_uuid_v7()));
    let mut service = bare(
        MockExpenseRepository::new(),
        categories_with(foreign.clone()),
    );

    let result = service
        .create_expense(command(org, foreign.id), now())
        .await;

    assert!(matches!(result, Err(common::CoreError::NotFound)));
}

#[tokio::test]
async fn an_advanced_expense_is_created_paid_by_card_on_its_date_without_due_date() {
    let org = OrganizationId(generate_uuid_v7());
    let category = category(org);
    let member_id = MemberId(generate_uuid_v7());
    let mut members = MockMemberRepository::new();
    members
        .expect_find_by_id()
        .with(eq(member_id))
        .returning(move |_| {
            Box::pin(async move {
                Ok(Some(Member {
                    id: member_id,
                    organization_id: org,
                    user_id: None,
                    last_name: "Claire".to_owned(),
                    first_name: None,
                    joined_at: None,
                    created_at: now(),
                    deleted_at: None,
                }))
            })
        });
    let mut service = make_service(
        inserting(),
        categories_with(category.clone()),
        MockSupplierRepository::new(),
        MockCustomerRepository::new(),
        members,
    );

    let created = service
        .create_expense(
            CreateExpenseCommand {
                advanced_by_member_id: Some(member_id),
                ..command(org, category.id)
            },
            now(),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(created.status, ExpenseStatus::Paid);
    assert_eq!(created.paid_on, Some(created.date));
    assert_eq!(created.method, Some(PaymentMethod::Card));
    assert_eq!(created.due_date, None);
    assert_eq!(created.reimbursed_on, None);
}

#[tokio::test]
async fn an_advancing_member_of_another_organization_is_not_found() {
    let org = OrganizationId(generate_uuid_v7());
    let category = category(org);
    let member_id = MemberId(generate_uuid_v7());
    let mut members = MockMemberRepository::new();
    members
        .expect_find_by_id()
        .returning(move |_| Box::pin(async { Ok(None) }));
    let mut service = make_service(
        MockExpenseRepository::new(),
        categories_with(category.clone()),
        MockSupplierRepository::new(),
        MockCustomerRepository::new(),
        members,
    );

    let result = service
        .create_expense(
            CreateExpenseCommand {
                advanced_by_member_id: Some(member_id),
                ..command(org, category.id)
            },
            now(),
        )
        .await;

    assert!(matches!(result, Err(common::CoreError::NotFound)));
}

#[tokio::test]
async fn a_supplier_or_customer_of_another_organization_is_not_found() {
    let org = OrganizationId(generate_uuid_v7());
    let category = category(org);
    let foreign_supplier = Supplier {
        id: SupplierId(generate_uuid_v7()),
        organization_id: OrganizationId(generate_uuid_v7()),
        name: "Point P".to_owned(),
        registration_number: None,
        vat_number: None,
        address: None,
        phone: None,
        email: None,
        profile: SupplierProfile::default(),
        deleted_at: None,
        created_at: now(),
        updated_at: now(),
    };
    let supplier_id = foreign_supplier.id;
    let mut suppliers = MockSupplierRepository::new();
    suppliers.expect_find_by_id().returning(move |_| {
        let supplier = foreign_supplier.clone();
        Box::pin(async move { Ok(Some(supplier)) })
    });
    let mut service = make_service(
        MockExpenseRepository::new(),
        categories_with(category.clone()),
        suppliers,
        MockCustomerRepository::new(),
        MockMemberRepository::new(),
    );
    let refused_supplier = service
        .create_expense(
            CreateExpenseCommand {
                supplier_id: Some(supplier_id),
                ..command(org, category.id)
            },
            now(),
        )
        .await;
    assert!(matches!(refused_supplier, Err(common::CoreError::NotFound)));

    let foreign_customer = Customer {
        id: CustomerId(generate_uuid_v7()),
        organization_id: OrganizationId(generate_uuid_v7()),
        status: CustomerStatus::Client,
        name: "Atelier".to_owned(),
        registration_number: None,
        phone: None,
        email: None,
        profile: CustomerProfile::default(),
        converted_at: None,
        archived_at: None,
        deleted_at: None,
        created_at: now(),
        updated_at: now(),
    };
    let customer_id = foreign_customer.id;
    let mut customers = MockCustomerRepository::new();
    customers.expect_find_by_id().returning(move |_| {
        let customer = foreign_customer.clone();
        Box::pin(async move { Ok(Some(customer)) })
    });
    let mut service = make_service(
        MockExpenseRepository::new(),
        categories_with(category.clone()),
        MockSupplierRepository::new(),
        customers,
        MockMemberRepository::new(),
    );
    let refused_customer = service
        .create_expense(
            CreateExpenseCommand {
                customer_id: Some(customer_id),
                ..command(org, category.id)
            },
            now(),
        )
        .await;
    assert!(matches!(refused_customer, Err(common::CoreError::NotFound)));
}

fn finding(existing: Expense) -> MockExpenseRepository {
    let mut expenses = MockExpenseRepository::new();
    expenses.expect_find_by_id().returning(move |_| {
        let expense = existing.clone();
        Box::pin(async move { Ok(Some(expense)) })
    });
    expenses
}

fn paid_command(id: ExpenseId) -> MarkExpensePaidCommand {
    MarkExpensePaidCommand {
        actor: Subject::system(),
        id,
        paid_on: date(9),
        method: PaymentMethod::Transfer,
    }
}

#[tokio::test]
async fn marking_paid_records_the_date_and_the_method() {
    let org = OrganizationId(generate_uuid_v7());
    let existing = expense(org);
    let id = existing.id;
    let mut expenses = finding(existing.clone());
    expenses
        .expect_mark_paid()
        .withf(|_, paid_on, method, _| *paid_on == date(9) && *method == PaymentMethod::Transfer)
        .times(1)
        .returning(move |_, paid_on, method, _| {
            let mut paid = existing.clone();
            paid.status = ExpenseStatus::Paid;
            paid.paid_on = Some(paid_on);
            paid.method = Some(method);
            Box::pin(async move { Ok(Some(paid)) })
        });
    let mut service = bare(expenses, MockExpenseCategoryRepository::new());

    let paid = service
        .mark_paid(paid_command(id), now())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(paid.status, ExpenseStatus::Paid);
    assert_eq!(paid.paid_on, Some(date(9)));
    assert_eq!(paid.method, Some(PaymentMethod::Transfer));
}

#[tokio::test]
async fn marking_paid_is_refused_on_a_paid_or_an_advanced_expense_and_writes_nothing() {
    let org = OrganizationId(generate_uuid_v7());
    let mut already_paid = expense(org);
    already_paid.status = ExpenseStatus::Paid;
    already_paid.paid_on = Some(date(2));
    already_paid.method = Some(PaymentMethod::Cash);
    let mut advanced = expense(org);
    advanced.advanced_by_member_id = Some(MemberId(generate_uuid_v7()));

    for existing in [already_paid, advanced] {
        let id = existing.id;
        let mut service = bare(finding(existing), MockExpenseCategoryRepository::new());

        let outcome = service.mark_paid(paid_command(id), now()).await.unwrap();

        assert_eq!(outcome.unwrap_err(), ExpenseError::NotPayable);
    }
}

#[tokio::test]
async fn reimbursing_works_once_on_an_advanced_expense() {
    let org = OrganizationId(generate_uuid_v7());
    let mut advanced = expense(org);
    advanced.advanced_by_member_id = Some(MemberId(generate_uuid_v7()));
    advanced.status = ExpenseStatus::Paid;
    advanced.due_date = None;
    advanced.paid_on = Some(advanced.date);
    advanced.method = Some(PaymentMethod::Card);
    let id = advanced.id;
    let calls = Arc::new(Mutex::new(0));
    let state = Arc::new(Mutex::new(advanced));

    let mut expenses = MockExpenseRepository::new();
    let read = state.clone();
    expenses.expect_find_by_id().returning(move |_| {
        let current = read.lock().unwrap().clone();
        Box::pin(async move { Ok(Some(current)) })
    });
    let write = state.clone();
    let counted = calls.clone();
    expenses.expect_reimburse().returning(move |_, on, _| {
        *counted.lock().unwrap() += 1;
        let mut current = write.lock().unwrap();
        current.reimbursed_on = Some(on);
        let updated = current.clone();
        Box::pin(async move { Ok(Some(updated)) })
    });
    let mut service = bare(expenses, MockExpenseCategoryRepository::new());

    let first = service
        .reimburse(id, Subject::system(), date(10), now())
        .await
        .unwrap()
        .unwrap();
    let second = service
        .reimburse(id, Subject::system(), date(11), now())
        .await
        .unwrap();

    assert_eq!(first.reimbursed_on, Some(date(10)));
    assert_eq!(second.unwrap_err(), ExpenseError::NotReimbursable);
    assert_eq!(*calls.lock().unwrap(), 1);
    assert_eq!(state.lock().unwrap().reimbursed_on, Some(date(10)));
}

#[tokio::test]
async fn reimbursing_a_non_advanced_expense_is_refused() {
    let org = OrganizationId(generate_uuid_v7());
    let existing = expense(org);
    let id = existing.id;
    let mut service = bare(finding(existing), MockExpenseCategoryRepository::new());

    let outcome = service
        .reimburse(id, Subject::system(), date(10), now())
        .await
        .unwrap();

    assert_eq!(outcome.unwrap_err(), ExpenseError::NotReimbursable);
}

#[tokio::test]
async fn updating_keeps_the_status_and_clears_the_due_date_of_an_advanced_expense() {
    let org = OrganizationId(generate_uuid_v7());
    let category = category(org);
    let mut advanced = expense(org);
    advanced.advanced_by_member_id = Some(MemberId(generate_uuid_v7()));
    advanced.status = ExpenseStatus::Paid;
    advanced.due_date = None;
    advanced.paid_on = Some(advanced.date);
    advanced.method = Some(PaymentMethod::Card);
    let id = advanced.id;
    let mut expenses = finding(advanced);
    expenses.expect_update().returning(|expense| {
        let expense = expense.clone();
        Box::pin(async move { Ok(expense) })
    });
    let mut service = bare(expenses, categories_with(category.clone()));

    let updated = service
        .update_expense(
            UpdateExpenseCommand {
                actor: Subject::system(),
                id,
                date: date(2),
                label: "Petit matériel".to_owned(),
                supplier_id: None,
                category_id: category.id,
                reference: None,
                amount_ht_cents: 5_400,
                vat_cents: 1_080,
                due_date: Some(date(30)),
                customer_id: None,
                proof_file_key: Some(" proofs/a.pdf ".to_owned()),
                note: None,
            },
            now(),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.status, ExpenseStatus::Paid);
    assert_eq!(updated.due_date, None);
    assert_eq!(updated.amount_ht_cents, 5_400);
    assert_eq!(updated.proof_file_key.as_deref(), Some("proofs/a.pdf"));
    assert!(updated.advanced_by_member_id.is_some());
}

#[test]
fn display_states_follow_advance_status_and_due_date() {
    let org = OrganizationId(generate_uuid_v7());
    let today = date(10);

    let to_pay = expense(org);
    assert_eq!(to_pay.display_state(today), ExpenseDisplayState::ToPay);

    let mut overdue = expense(org);
    overdue.due_date = Some(date(7));
    assert_eq!(overdue.display_state(today), ExpenseDisplayState::Overdue);
    assert_eq!(overdue.days_late(today), 3);

    let mut due_today = expense(org);
    due_today.due_date = Some(today);
    assert_eq!(due_today.display_state(today), ExpenseDisplayState::ToPay);

    let mut paid = expense(org);
    paid.status = ExpenseStatus::Paid;
    paid.due_date = Some(date(1));
    assert_eq!(paid.display_state(today), ExpenseDisplayState::Paid);

    let mut to_reimburse = expense(org);
    to_reimburse.advanced_by_member_id = Some(MemberId(generate_uuid_v7()));
    assert_eq!(
        to_reimburse.display_state(today),
        ExpenseDisplayState::ToReimburse
    );
    to_reimburse.reimbursed_on = Some(today);
    assert_eq!(
        to_reimburse.display_state(today),
        ExpenseDisplayState::Reimbursed
    );
}

fn category_service(
    categories: MockExpenseCategoryRepository,
) -> ExpenseCategoryService<
    MockExpenseCategoryRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
> {
    ExpenseCategoryService::new(
        categories,
        MockMemberRepository::new(),
        MockRoleRepository::new(),
        allow_all(),
    )
}

#[tokio::test]
async fn an_organization_without_category_gets_the_defaults_once() {
    let org = OrganizationId(generate_uuid_v7());
    let stored: Arc<Mutex<Vec<ExpenseCategory>>> = Arc::default();
    let mut categories = MockExpenseCategoryRepository::new();
    let read = stored.clone();
    categories
        .expect_list_by_organization()
        .returning(move |_| {
            let all = read.lock().unwrap().clone();
            Box::pin(async move { Ok(all) })
        });
    let write = stored.clone();
    categories.expect_insert().returning(move |category| {
        write.lock().unwrap().push(category.clone());
        let category = category.clone();
        Box::pin(async move { Ok(category) })
    });
    let mut service = category_service(categories);

    let first = service.list_categories(org, now()).await.unwrap();
    let second = service.list_categories(org, now()).await.unwrap();

    assert_eq!(first.len(), DEFAULT_CATEGORIES.len());
    assert_eq!(second.len(), DEFAULT_CATEGORIES.len());
    assert!(first.iter().all(|category| category.organization_id == org));
}

#[tokio::test]
async fn a_category_needs_a_name_and_a_hue_below_360_and_a_free_name() {
    let org = OrganizationId(generate_uuid_v7());
    let existing = category(org);
    let mut categories = MockExpenseCategoryRepository::new();
    categories
        .expect_list_by_organization()
        .returning(move |_| {
            let existing = existing.clone();
            Box::pin(async move { Ok(vec![existing]) })
        });
    let mut service = category_service(categories);
    let command = |name: &str, hue: i64| CreateExpenseCategoryCommand {
        actor: Subject::system(),
        organization_id: org,
        name: name.to_owned(),
        hue,
    };

    let blank = service
        .create_category(command("  ", 10), now())
        .await
        .unwrap();
    let hue = service
        .create_category(command("Autres", 360), now())
        .await
        .unwrap();
    let taken = service
        .create_category(command("matériaux", 10), now())
        .await
        .unwrap();

    assert_eq!(blank.unwrap_err(), ExpenseError::NameRequired);
    assert_eq!(hue.unwrap_err(), ExpenseError::HueOutOfRange);
    assert_eq!(taken.unwrap_err(), ExpenseError::CategoryNameTaken);
}

#[tokio::test]
async fn a_category_used_by_an_expense_cannot_be_deleted() {
    let org = OrganizationId(generate_uuid_v7());
    let existing = category(org);
    let id = existing.id;
    let mut categories = categories_with(existing);
    categories
        .expect_is_used()
        .returning(|_| Box::pin(async { Ok(true) }));
    let mut service = category_service(categories);

    let outcome = service
        .delete_category(id, Subject::system())
        .await
        .unwrap();

    assert_eq!(outcome.unwrap_err(), ExpenseError::CategoryInUse);
}
