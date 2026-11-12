Feature: Work order lifecycle
  An admin creates a work order for an agent or for a label. An agent that
  the work order reaches sees it pending, claims it, and completes it. A
  failure waits and comes back until its retries are spent. A claim that
  passes its timeout goes back to pending. The fleet record counts the
  pending and the claimed work orders of the agent. The scenario plays the
  agent with the agent's PAK.

  Background:
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the generator "team-robin"

  Scenario: A work order goes from pending to the log as a success
    Given the admin has created the work order "backup" for the agent "prod-1"
    Then the agent "prod-1" has the work order "backup" pending
    And the fleet record of the agent "prod-1" counts 1 pending and 0 claimed work orders
    When the agent "prod-1" claims the work order "backup"
    Then the work order "backup" is CLAIMED
    And the fleet record of the agent "prod-1" counts 0 pending and 1 claimed work orders
    When the agent "prod-1" completes the work order "backup"
    Then the log has the work order "backup" as a success
    And the fleet record of the agent "prod-1" counts 0 pending and 0 claimed work orders

  Scenario: A work order that fails comes back, and ends in the log as a failure
    Given the admin has created the work order "backup" for the agent "prod-1" with 1 retry
    When the agent "prod-1" claims the work order "backup"
    And the agent "prod-1" fails the work order "backup"
    Then the work order "backup" is RETRY_PENDING
    And the work order "backup" is PENDING within 30 seconds
    When the agent "prod-1" claims the work order "backup"
    And the agent "prod-1" fails the work order "backup"
    Then the log has the work order "backup" as a failure

  Scenario: A claim that passes its timeout goes back to pending
    Given the admin has created the work order "backup" for the agent "prod-1" with a claim timeout of 1 second
    When the agent "prod-1" claims the work order "backup"
    Then the work order "backup" is PENDING within 30 seconds
    And the agent "prod-1" has the work order "backup" pending

  Scenario: A work order for a label reaches each agent with the label, and one agent claims it
    Given the admin has created the agent "prod-2" in cluster "cluster-a" for the generator "team-robin"
    And the admin has given the agent "prod-1" the label "pool" of this scenario
    And the admin has given the agent "prod-2" the label "pool" of this scenario
    And the admin has created the work order "backup" for the label "pool" of this scenario
    Then the agent "prod-1" has the work order "backup" pending
    And the agent "prod-2" has the work order "backup" pending
    When the agent "prod-1" claims the work order "backup"
    Then the agent "prod-2" does not have the work order "backup" pending
    When the agent "prod-2" tries to claim the work order "backup"
    Then the broker says the work order is not claimable

  Scenario: A work order for a deleted agent is refused
    Given the admin has deleted the agent "prod-1"
    When the admin creates the work order "backup" for the agent "prod-1"
    Then the broker does not accept the work order

  Scenario: A pending work order does not stay pending when its only agent is deleted
    Given the admin has created the work order "backup" for the agent "prod-1"
    When the admin deletes the agent "prod-1"
    Then the work order "backup" is not left PENDING within 20 seconds
