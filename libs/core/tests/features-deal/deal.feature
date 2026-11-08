Feature: Deals, the sales opportunities of a customer

  A deal is an opportunity of a customer. It starts as a new request, moves
  through the open stages, and ends won or lost. Winning turns a prospect into a
  client, losing needs a reason, and archiving a customer loses its open deals.

  Background:
    Given a company registered on Mestier
    And a customer "Boulangerie Lemoine" that is a PROSPECT

  @spec-crm-3-1
  Scenario: A new deal starts at NEW
    When the artisan creates the deal "Rénovation du fournil" of 1250000 cents for "Boulangerie Lemoine"
    Then the deal "Rénovation du fournil" is in stage NEW
    And the deal "Rénovation du fournil" is open
    And the deal "Rénovation du fournil" is worth 1250000 cents
    And the company is notified of the event "deal.created"
    And the event "deal.created" carries "/deal/title" equal to "Rénovation du fournil"
    And the event "deal.created" carries "/deal/stage" equal to "NEW"

  @spec-crm-3-1
  Scenario Outline: A deal without a title, a known customer or a positive amount is refused
    When the artisan creates the deal "<title>" of <amount> cents for "<customer>"
    Then the deal is refused with "<refusal>"
    And no deal is stored
    And the company is notified of no deal event

    Examples:
      | title     | amount | customer            | refusal                            |
      |           | 1000   | Boulangerie Lemoine | title: must not be empty           |
      | Fournil   | -1     | Boulangerie Lemoine | amount_cents: must not be negative |
      | Fournil   | 1000   | Inconnu             | not found                          |

  @spec-crm-3-2
  Scenario: Moving a deal to LOST without a reason changes nothing
    Given a deal "Rénovation du fournil" in stage NEGOTIATION for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to LOST without a reason
    Then the deal is refused with "lost_reason: a lost deal needs a reason"
    And the deal "Rénovation du fournil" is in stage NEGOTIATION
    And the deal "Rénovation du fournil" is open

  @spec-crm-3-2
  Scenario: Moving a deal to LOST with a reason sets the reason and the close date
    Given a deal "Rénovation du fournil" in stage NEGOTIATION for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to LOST with the reason "Prix trop élevé"
    Then the deal "Rénovation du fournil" is in stage LOST
    And the deal "Rénovation du fournil" has the lost reason "Prix trop élevé"
    And the deal "Rénovation du fournil" is closed today

  @spec-crm-3-3
  Scenario: Winning a deal closes it and converts a prospect
    Given a deal "Rénovation du fournil" in stage NEGOTIATION for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to WON
    Then the deal "Rénovation du fournil" is in stage WON
    And the deal "Rénovation du fournil" is closed today
    And the customer "Boulangerie Lemoine" is a CLIENT
    And the customer "Boulangerie Lemoine" was converted today

  @spec-crm-3-3
  Scenario: Winning a deal of a client keeps its first conversion date
    Given a customer "Ancien client" converted 200 days ago
    And a deal "Extension" in stage VISIT for "Ancien client"
    When the artisan moves the deal "Extension" to WON
    Then the deal "Extension" is in stage WON
    And the customer "Ancien client" was converted 200 days ago

  @spec-crm-3-4
  Scenario: Reopening a lost deal clears the close date and the reason
    Given a deal "Rénovation du fournil" lost with the reason "Pas de réponse" for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to NEGOTIATION
    Then the deal "Rénovation du fournil" is in stage NEGOTIATION
    And the deal "Rénovation du fournil" is open
    And the deal "Rénovation du fournil" has no lost reason

  @spec-crm-3-4
  Scenario: Reopening a won deal clears the close date
    Given a deal "Rénovation du fournil" in stage WON for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to VISIT
    Then the deal "Rénovation du fournil" is in stage VISIT
    And the deal "Rénovation du fournil" is open

  @spec-crm-3-5
  Scenario: A deal that has a quote cannot be deleted
    Given a deal "Rénovation du fournil" in stage QUOTE_SENT for "Boulangerie Lemoine"
    And the deal "Rénovation du fournil" has a quote
    When the artisan deletes the deal "Rénovation du fournil"
    Then the deal is refused with "quotes: a deal linked to a quote cannot be deleted"
    And the deal "Rénovation du fournil" still exists

  @spec-crm-3-5
  Scenario: A deal without a quote can be deleted
    Given a deal "Rénovation du fournil" in stage NEW for "Boulangerie Lemoine"
    When the artisan deletes the deal "Rénovation du fournil"
    Then the deal "Rénovation du fournil" no longer exists

  @spec-crm-3-6
  Scenario: Archiving a customer loses its open deals and keeps the won ones
    Given a deal "Cuisine" in stage NEW for "Boulangerie Lemoine"
    And a deal "Fournil" in stage QUOTE_SENT for "Boulangerie Lemoine"
    And a deal "Vitrine" in stage WON for "Boulangerie Lemoine"
    When the artisan archives the customer "Boulangerie Lemoine"
    Then the deal "Cuisine" is in stage LOST
    And the deal "Cuisine" has the lost reason "Compte archivé"
    And the deal "Cuisine" is closed today
    And the deal "Fournil" has the lost reason "Compte archivé"
    And the deal "Vitrine" is in stage WON
    And the deal "Vitrine" has no lost reason

  @spec-crm-3-6
  Scenario: Archiving a customer leaves the deals of another customer alone
    Given a customer "Autre client" that is a CLIENT
    And a deal "Cuisine" in stage NEW for "Boulangerie Lemoine"
    And a deal "Terrasse" in stage VISIT for "Autre client"
    When the artisan archives the customer "Boulangerie Lemoine"
    Then the deal "Terrasse" is in stage VISIT

  @spec-crm-3-7
  Scenario Outline: The summary counts won and lost deals of the last 90 days
    Given <won> deals won within the last 90 days
    And <lost> deals lost within the last 90 days
    And a deal won 91 days ago
    And a deal lost 91 days ago
    When the artisan reads the pipeline summary
    Then the summary counts <won> won and <lost> lost deals
    And the win rate is <rate>

    Examples:
      | won | lost | rate |
      | 1   | 2    | 33   |
      | 2   | 1    | 67   |
      | 3   | 0    | 100  |
      | 0   | 4    | 0    |
      | 0   | 0    | none |

  @spec-crm-3-7
  Scenario: A deal closed exactly 90 days ago is counted
    Given a deal won 90 days ago
    When the artisan reads the pipeline summary
    Then the summary counts 1 won and 0 lost deals
    And the win rate is 100

  @spec-crm-3-7
  Scenario: The open pipeline counts the four open stages only
    Given a deal "A" in stage NEW for "Boulangerie Lemoine"
    And a deal "B" in stage NEGOTIATION for "Boulangerie Lemoine"
    And a deal "C" in stage WON for "Boulangerie Lemoine"
    When the artisan reads the pipeline summary
    Then the open pipeline holds 2 deals

  @spec-crm-3-8
  Scenario: The summary counts the customers converted in the last 90 days
    Given a customer "Récent" converted 10 days ago
    And a customer "Limite" converted 90 days ago
    And a customer "Ancien" converted 91 days ago
    When the artisan reads the pipeline summary
    Then the summary counts 2 conversions

  @spec-crm-3-7
  Scenario: Moving a deal announces the stage it left and the stage it reached
    Given a deal "Rénovation du fournil" in stage NEW for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to VISIT
    Then the company is notified of 1 events "deal.stage_changed"
    And the event "deal.stage_changed" carries "/from" equal to "NEW"
    And the event "deal.stage_changed" carries "/to" equal to "VISIT"
    And the event "deal.stage_changed" carries "/deal/stage" equal to "VISIT"

  @spec-crm-3-7
  Scenario: A lost deal announces its reason
    Given a deal "Rénovation du fournil" in stage NEGOTIATION for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to LOST with the reason "Prix trop élevé"
    Then the event "deal.stage_changed" carries "/to" equal to "LOST"
    And the event "deal.stage_changed" carries "/deal/lost_reason" equal to "Prix trop élevé"

  @spec-crm-3-7
  Scenario: Moving a deal to the stage it is already in announces nothing
    Given a deal "Rénovation du fournil" in stage VISIT for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to VISIT
    Then the company is notified of no deal event

  @spec-crm-3-7
  Scenario: A refused move announces nothing
    Given a deal "Rénovation du fournil" in stage NEGOTIATION for "Boulangerie Lemoine"
    When the artisan moves the deal "Rénovation du fournil" to LOST without a reason
    Then the deal is refused with "lost_reason: a lost deal needs a reason"
    And the company is notified of no deal event

  @spec-crm-3-7
  Scenario: Archiving a customer announces each deal it loses
    Given a deal "Cuisine" in stage NEW for "Boulangerie Lemoine"
    And a deal "Fournil" in stage QUOTE_SENT for "Boulangerie Lemoine"
    And a deal "Vitrine" in stage WON for "Boulangerie Lemoine"
    When the artisan archives the customer "Boulangerie Lemoine"
    Then the company is notified of 2 events "deal.stage_changed"
