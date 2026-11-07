Feature: Pricing, sending and answering a quote

  A quote is the first link in the profitability loop. The artisan enters lines
  of work, and the software derives the reference, the total and the status from
  them: never the other way round. That total is the single source of truth,
  used by the screen and by the PDF sent to the customer alike.

  Background:
    Given a company registered on Mestier
    And a customer "Duval Masonry" belonging to that company
    And today is "2026-11-07"

  Scenario: The total of a quote is computed from its lines
    Given a company that has not issued any quote in 2026
    When the artisan creates the quote "Kitchen renovation" with the following lines:
      | work                  | quantity | unit | unit price |
      | Strip out existing    | 4        | hour | €45.00     |
      | Lay wall tiles        | 12.5     | m2   | €38.00     |
      | Fit skirting boards   | 8        | ml   | €27.50     |
    Then the quote has no reference yet
    And the total of the quote is "€875.00"
    And the quote is in status "draft"
    And the company is notified of the event "quote.created"

  Scenario: A reference is allocated only when the quote is sent, and never again afterwards
    Given a company that has not issued any quote in 2026
    When the artisan creates the quote "Kitchen renovation" with the following lines:
      | work               | quantity | unit | unit price |
      | Strip out existing | 4        | hour | €45.00     |
    And the artisan sends the quote
    Then the quote carries the reference "DEV-2026-0001"
    When the artisan sends the quote again
    Then the quote carries the reference "DEV-2026-0001"

  Scenario: A quote accepted by the customer changes status and notifies the company
    Given a quote "Kitchen renovation" in status "sent"
    When the customer accepts the quote
    Then the quote is in status "accepted"
    And the company is notified of the event "quote.accepted"

  Scenario: A line with no billable quantity is refused
    When the artisan creates the quote "Sloppy quote" with the following lines:
      | work           | quantity | unit | unit price |
      | Lay wall tiles | 0        | m2   | €38.00     |
    Then the creation is refused
    And no quote was stored

  @spec-crm-4-1
  Scenario: A quote is editable only as a draft and only a draft can be deleted
    Given a quote "Kitchen renovation" in status "sent"
    When the artisan renames the quote to "Bathroom"
    Then the answer is refused with the code "E_QUOTE_LOCKED"
    And the quote is titled "Kitchen renovation"
    When the artisan deletes the quote
    Then the answer is refused with the code "E_QUOTE_LOCKED"
    And no reference was allocated

  @spec-crm-4-1
  Scenario: Deleting a draft uses no number
    Given a quote "Kitchen renovation" in status "draft"
    When the artisan deletes the quote
    Then no quote is left
    And no reference was allocated

  @spec-crm-4-2
  Scenario: Sending is refused without a billing street and city, and consumes no number
    Given a customer without a billing address
    And a quote "Kitchen renovation" in status "draft"
    When the artisan sends the quote
    Then the answer is refused with the code "E_UNPROCESSABLE_ENTITY" on "billing_address"
    And no reference was allocated
    And the quote is in status "draft"

  @spec-crm-4-2
  Scenario: Sending is refused when valid until is in the past
    Given a quote "Kitchen renovation" in status "draft" valid until "2026-11-06"
    When the artisan sends the quote
    Then the answer is refused with the code "E_UNPROCESSABLE_ENTITY" on "valid_until"
    And no reference was allocated

  @spec-crm-4-3
  Scenario: Sending assigns the number, the validity and freezes both parties
    Given the company keeps its quotes valid 15 days
    And a quote "Kitchen renovation" in status "draft"
    When the artisan sends the quote
    Then the quote carries the reference "DEV-2026-0001"
    And the quote is valid until "2026-11-22"
    And the quote froze the issuer "Scenario SARL" and the recipient "Duval Masonry"

  @spec-crm-4-5
  Scenario: A sent quote is expired only the day after its validity ends
    Given a quote "Kitchen renovation" in status "sent" valid until "2026-11-07"
    Then the quote is not expired
    Given today is "2026-11-08"
    Then the quote is expired

  @spec-crm-4-6
  Scenario: Accepting records who accepted and when
    Given a quote "Kitchen renovation" in status "sent" valid until "2026-11-30"
    When the customer "  Mme Duval " accepts the quote
    Then the quote is in status "accepted"
    And the quote was answered by "Mme Duval" on "2026-11-07"

  @spec-crm-4-6
  Scenario: Accepting an expired quote is refused
    Given a quote "Kitchen renovation" in status "sent" valid until "2026-11-01"
    When the customer accepts the quote
    Then the answer is refused with the code "E_QUOTE_EXPIRED"
    And the quote is in status "sent"

  @spec-crm-4-6
  Scenario: Accepting needs a sent quote
    Given a quote "Kitchen renovation" in status "draft"
    When the customer accepts the quote
    Then the answer is refused with the code "E_QUOTE_NOT_SENT"

  @spec-crm-4-7
  Scenario: Declining records the reason
    Given a quote "Kitchen renovation" in status "sent"
    When the customer declines the quote because " Too expensive "
    Then the quote is in status "declined"
    And the quote was declined because "Too expensive"

  @spec-crm-4-8
  Scenario: A revision copies the quote as a draft without number nor validity
    Given a quote "Kitchen renovation" in status "declined" valid until "2026-11-30"
    When the artisan revises the quote
    Then the revision is a draft without reference nor validity
    And the revision keeps the title "Kitchen renovation" and the lines of the source

  @spec-crm-4-9
  Scenario: Sending a revision supersedes the source that is still sent
    Given a quote "Kitchen renovation" in status "sent"
    When the artisan revises the quote
    And the artisan sends the revision
    Then the source quote is in status "superseded"

  @spec-crm-4-9
  Scenario: Sending a revision leaves a declined source declined
    Given a quote "Kitchen renovation" in status "declined"
    When the artisan revises the quote
    And the artisan sends the revision
    Then the source quote is in status "declined"

  @spec-crm-4-10
  Scenario: A line discount applies before VAT
    Given a company subject to VAT
    When the artisan creates the quote "Kitchen renovation" with the following lines:
      | work           | quantity | unit | unit price | discount | vat   |
      | Lay wall tiles | 3        | m2   | €100.00    | 10       | 20.00 |
    Then the total of the quote is "€270.00"
    And the VAT of the quote is "€54.00"

  @spec-crm-4-11
  Scenario: The VAT is computed once per rate on the summed base
    Given a company subject to VAT
    When the artisan creates the quote "Kitchen renovation" with the following lines:
      | work    | quantity | unit | unit price | vat   |
      | Pose    | 1        | unit | €100.33    | 20.00 |
      | Fixings | 1        | unit | €100.33    | 20.00 |
    Then the total of the quote is "€200.66"
    And the VAT of the quote is "€40.13"

  @spec-crm-4-12
  Scenario: Frequency leaves the totals alone and lists the recurring totals
    Given a company subject to VAT
    When the artisan creates the quote "Maintenance" with the following lines:
      | work        | quantity | unit | unit price | vat   | frequency |
      | Installation | 1       | unit | €1000.00   | 20.00 | one_shot  |
      | Upkeep      | 1        | unit | €20.00     | 20.00 | monthly   |
    Then the total of the quote is "€1020.00"
    And the recurring totals are "MONTHLY 2000 400 2400"

  @spec-crm-4-14
  Scenario: The customer of a quote that has lines cannot change
    Given a quote "Kitchen renovation" in status "draft"
    When the artisan moves the quote to another customer
    Then the answer is refused with the code "E_UNPROCESSABLE_ENTITY" on "customer_id"
