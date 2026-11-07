Feature: Customer profile, lifecycle and billing designation

  A customer carries the facts the CRM screens show: its kind, website, VAT
  number, owner, lead source, notes, payment terms and the contact and address
  invoices are sent to. Its status moves from prospect to client to archived,
  and the dates of those moves are kept.

  Background:
    Given a company registered on Mestier
    And a team member "Claire"

  @spec-crm-2-1
  Scenario: A customer stores its profile
    When the artisan creates the PROFESSIONAL customer "Boulangerie Lemoine" with website "https://lemoine.fr", VAT number "FR12345678901", owner "Claire", source "salon", notes "Livraison le matin" and payment terms of 45 days
    Then the customer is a PROFESSIONAL
    And the customer website is "https://lemoine.fr"
    And the customer VAT number is "FR12345678901"
    And the customer owner is "Claire"
    And the customer source is "salon"
    And the customer notes are "Livraison le matin"
    And the customer payment terms are 45 days

  @spec-crm-2-1
  Scenario: A customer designates its billing contact and billing address
    Given a customer "Boulangerie Lemoine" with a contact "Marc" and an address "Siège"
    When the artisan designates the contact "Marc" and the address "Siège" for billing
    Then the billing contact is "Marc"
    And the billing address is "Siège"

  @spec-crm-2-2
  Scenario: Converting a prospect sets converted_at once and keeps the first date
    Given a customer "Boulangerie Lemoine" that is a PROSPECT
    When the artisan sets the status to CLIENT
    Then the customer status is CLIENT
    And converted_at is set
    When the artisan sets the status to CLIENT
    Then converted_at is the same as after the first conversion

  @spec-crm-2-3
  Scenario: Archiving sets archived_at, restoring returns to CLIENT when the customer was converted
    Given a customer "Boulangerie Lemoine" that is a CLIENT
    When the artisan sets the status to ARCHIVED
    Then the customer status is ARCHIVED
    And archived_at is set
    When the artisan sets the status to PROSPECT
    Then the customer status is CLIENT
    And archived_at is empty

  @spec-crm-2-3
  Scenario: Restoring a customer that was never converted returns to PROSPECT
    Given a customer "Boulangerie Lemoine" that is a PROSPECT
    When the artisan sets the status to ARCHIVED
    And the artisan sets the status to PROSPECT
    Then the customer status is PROSPECT
    And converted_at is empty
    And archived_at is empty

  @spec-crm-2-4
  Scenario Outline: A registration number of 9 or 14 digits is accepted, spaces ignored
    When the artisan enters the registration number "<entered>"
    Then the registration number is stored as "<stored>"

    Examples:
      | entered           | stored         |
      | 123456789         | 123456789      |
      | 123 456 789       | 123456789      |
      | 123 456 789 00012 | 12345678900012 |

  @spec-crm-2-4
  Scenario Outline: Any other length is refused, naming the field
    When the artisan enters the registration number "<entered>"
    Then the registration number is refused on the field "registration_number"

    Examples:
      | entered         |
      | 12345678        |
      | 1234567890      |
      | 123456789000123 |

  @spec-crm-2-6
  Scenario: A contact stores a mobile and a phone
    Given a customer "Boulangerie Lemoine" that is a PROSPECT
    When the artisan adds the contact "Marc" with mobile "06 12 34 56 78" and phone "02 40 00 00 00"
    Then the contact "Marc" has the mobile "06 12 34 56 78" and the phone "02 40 00 00 00"

  @spec-crm-2-6
  Scenario: An address stores its kind, country and second line
    Given a customer "Boulangerie Lemoine" that is a PROSPECT
    When the artisan adds the BILLING address "Siège" in "BE" with second line "Bâtiment B"
    Then the address "Siège" is of kind BILLING in "BE" with second line "Bâtiment B"

  @spec-crm-2-7
  Scenario: A billing contact from another customer is refused
    Given a customer "Boulangerie Lemoine" that is a PROSPECT
    And a customer "Garage Durand" with a contact "Paul" and an address "Atelier"
    When the artisan designates the contact "Paul" of "Garage Durand" for billing on "Boulangerie Lemoine"
    Then the designation is refused on the field "billing_contact_id"
    And the customer "Boulangerie Lemoine" has no billing contact

  @spec-crm-2-7
  Scenario: A billing address from another customer is refused
    Given a customer "Boulangerie Lemoine" that is a PROSPECT
    And a customer "Garage Durand" with a contact "Paul" and an address "Atelier"
    When the artisan designates the address "Atelier" of "Garage Durand" for billing on "Boulangerie Lemoine"
    Then the designation is refused on the field "billing_address_id"
    And the customer "Boulangerie Lemoine" has no billing address

  @spec-crm-2-7
  Scenario: An owner who is not a member of the company is refused
    When the artisan creates a customer owned by a member of another company
    Then the designation is refused on the field "owner_member_id"

  @spec-crm-2-8
  Scenario Outline: Payment terms outside 0 to 60 days are refused, naming the field
    When the artisan enters payment terms of <days> days
    Then the payment terms are refused on the field "payment_terms_days"

    Examples:
      | days |
      | 61   |
      | -1   |
