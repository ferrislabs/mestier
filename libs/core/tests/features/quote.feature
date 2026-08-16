Feature: Pricing and accepting a quote

  A quote is the first link in the profitability loop. The artisan enters lines
  of work, and the software derives the reference, the total and the status from
  them: never the other way round. That total is the single source of truth,
  used by the screen and by the PDF sent to the customer alike.

  Background:
    Given a company registered on Mestier
    And a customer "Duval Masonry" belonging to that company

  Scenario: The total of a quote is computed from its lines
    Given a company that has not issued any quote in 2026
    When the artisan creates the quote "Kitchen renovation" with the following lines:
      | work                  | quantity | unit | unit price |
      | Strip out existing    | 4        | hour | €45.00     |
      | Lay wall tiles        | 12.5     | m2   | €38.00     |
      | Fit skirting boards   | 8        | ml   | €27.50     |
    Then the quote carries the reference "DEV-2026-0001"
    And the total of the quote is "€875.00"
    And the quote is in status "draft"
    And the company is notified of the event "quote.created"

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
