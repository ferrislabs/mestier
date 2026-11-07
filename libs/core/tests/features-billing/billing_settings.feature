Feature: Billing settings and number series of an organization

  An organization sets its payment terms, quote validity, default VAT rate,
  late-payment mentions and counter padding once. Each document series keeps
  its own counter, per year, that only moves forward.

  Background:
    Given a company registered on Mestier

  @spec-crm-1-1
  Scenario: Reading the settings of an organization that has none returns the defaults
    When the artisan reads the billing settings
    Then the payment terms are 30 days
    And the quote validity is 30 days
    And the default VAT rate is 2000 basis points
    And the recovery fee is 4000 cents
    And the number padding is 4
    And no IBAN is set

  @spec-crm-1-2
  Scenario Outline: A setting out of its bounds is refused, naming the field
    When the artisan saves the billing settings with <field> set to <value>
    Then the save is refused on the field "<field>"
    And no billing settings were stored

    Examples:
      | field               | value |
      | payment_terms_days  | 61    |
      | number_padding      | 7     |
      | quote_validity_days | 0     |
      | recovery_fee_cents  | -1    |

  @spec-crm-1-3
  Scenario Outline: The next number of a series cannot be set at or below the last assigned one
    Given the <series> series has assigned number 10 in 2026
    When the artisan sets the next <series> number to <requested> for 2026
    Then the change is refused as not forward, the last assigned number being 10
    And the <series> series still has 10 as last assigned number in 2026

    Examples:
      | series      | requested |
      | QUOTE       | 10        |
      | INVOICE     | 4         |
      | CREDIT_NOTE | 1         |

  @spec-crm-1-3
  Scenario: The next number of a series can be set above the last assigned one
    Given the INVOICE series has assigned number 10 in 2026
    When the artisan sets the next INVOICE number to 120 for 2026
    Then the INVOICE series has 119 as last assigned number in 2026
    And the next INVOICE reference of 2026 is "FAC-2026-0120"

  @spec-crm-1-5
  Scenario: A new padding applies to the next number only
    Given the CREDIT_NOTE series has assigned number 7 in 2026
    And the reference "AV-2026-0007" was assigned before
    When the artisan saves the billing settings with number_padding set to 6
    Then the next CREDIT_NOTE reference of 2026 is "AV-2026-000008"
    And the reference "AV-2026-0007" is unchanged
