use authz::{Decision, MockAuthorizer, Subject};
use chrono::{NaiveDate, Utc};
use common::{CoreError, UserId, generate_uuid_v7};

use crate::{
    LedgerError, MockCustomerRepository, MockMemberRepository, MockReceiptRepository,
    MockRoleRepository, OrganizationId, PaymentMethod, ReceiptService,
    domain::ledger::commands::RecordReceiptCommand,
};

fn allow_all() -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    authz
}

fn command(amount_cents: i32, paid_on: NaiveDate) -> RecordReceiptCommand {
    RecordReceiptCommand {
        actor: Subject::system(),
        organization_id: OrganizationId(generate_uuid_v7()),
        customer_id: None,
        amount_cents,
        paid_on,
        method: PaymentMethod::Check,
        reference: Some("  CHQ 42 ".to_owned()),
        note: Some("   ".to_owned()),
        recorded_by: UserId(generate_uuid_v7()),
    }
}

fn service(
    receipts: MockReceiptRepository,
) -> ReceiptService<
    MockReceiptRepository,
    MockCustomerRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
> {
    ReceiptService::new(
        receipts,
        MockCustomerRepository::new(),
        MockMemberRepository::new(),
        MockRoleRepository::new(),
        allow_all(),
    )
}

#[tokio::test]
async fn a_receipt_needs_a_positive_amount_and_a_date_not_in_the_future() {
    let now = Utc::now();
    let today = now.date_naive();
    let mut service = service(MockReceiptRepository::new());

    let zero = service
        .record_receipt(command(0, today), now)
        .await
        .unwrap();
    let negative = service
        .record_receipt(command(-5, today), now)
        .await
        .unwrap();
    let future = service
        .record_receipt(command(100, today.succ_opt().unwrap()), now)
        .await
        .unwrap();

    assert_eq!(zero.unwrap_err(), LedgerError::AmountNotPositive);
    assert_eq!(negative.unwrap_err(), LedgerError::AmountNotPositive);
    assert_eq!(future.unwrap_err(), LedgerError::PaidOnInFuture);
}

#[tokio::test]
async fn a_receipt_is_stored_unallocated_with_trimmed_texts() {
    let now = Utc::now();
    let mut receipts = MockReceiptRepository::new();
    receipts.expect_insert().times(1).returning(|receipt| {
        let receipt = receipt.clone();
        Box::pin(async move { Ok(receipt) })
    });
    let mut service = service(receipts);

    let receipt = service
        .record_receipt(command(12_500, now.date_naive()), now)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(receipt.amount_cents, 12_500);
    assert_eq!(receipt.reference.as_deref(), Some("CHQ 42"));
    assert_eq!(receipt.note, None);
}

#[tokio::test]
async fn an_unknown_receipt_is_not_found() {
    let mut receipts = MockReceiptRepository::new();
    receipts
        .expect_find_by_id()
        .returning(|_| Box::pin(async { Ok(None) }));
    let mut service = service(receipts);

    let result = service
        .get_receipt(crate::ReceiptId(generate_uuid_v7()))
        .await;

    assert!(matches!(result, Err(CoreError::NotFound)));
}
