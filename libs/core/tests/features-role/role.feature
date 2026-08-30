Feature: Role-based permissions

  Roles bundle permission bits and are assigned to members; a member's
  effective permissions are the union (bitwise OR) of every role they hold.
  Three roles are seeded when an organization is created (owner, admin,
  member) and are protected: their name is fixed and they cannot be deleted,
  though their permissions stay editable. A custom role can be freely
  renamed, re-permissioned, assigned, unassigned and deleted — but never
  while still held by a member.

  Scenario: A role is created with a chosen subset of permissions
    Given an organization
    When a role "Accountant" is created with permissions "VIEW_COST, VIEW_REPORTS"
    Then the role "Accountant" holds exactly the permissions "VIEW_COST, VIEW_REPORTS"

  Scenario: A member holding two roles gets the union of their permissions
    Given a member with no role
    And a role "Planner" with permissions "VIEW_PLANNING, MANAGE_PLANNING"
    And a role "Accountant" with permissions "VIEW_COST, VIEW_REPORTS"
    When the member is assigned the role "Planner"
    And the member is assigned the role "Accountant"
    Then the member's aggregated permissions are exactly "VIEW_PLANNING, MANAGE_PLANNING, VIEW_COST, VIEW_REPORTS"

  Scenario: Unassigning one role does not affect the other
    Given a member holding the roles "Planner" and "Accountant"
    When the role "Planner" is unassigned from the member
    Then the member's aggregated permissions are exactly "VIEW_COST, VIEW_REPORTS"

  Scenario: Unassigning a role a member never held is not an error
    Given a member with no role
    And a role "Accountant" with permissions "VIEW_COST, VIEW_REPORTS"
    When the role "Accountant" is unassigned from the member
    Then the member's aggregated permissions are exactly ""

  Scenario: A seeded role's name cannot change
    Given the seeded role "admin"
    When renaming the role "admin" to "Administrator" is attempted
    Then the attempt is refused

  Scenario: A role still assigned to a member cannot be deleted
    Given a member holding the role "Accountant"
    When deleting the role "Accountant" is attempted
    Then the attempt is refused
