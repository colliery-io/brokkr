Feature: Console
  The operator console that the broker serves shows the fleet, the health of
  each agent and the active work orders. The scenarios set up a state with
  the steps of the lifecycle features, then read the console with a browser.
  The console reads the broker every few seconds, so each check waits until
  the view shows the state. The scenario plays the agent with the agent's PAK.

  Background:
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in a cluster of its own for the generator "team-robin"
    And the admin has activated the agent "prod-1"

  Scenario: The Fleet view shows an active agent that talks as healthy
    When the agent "prod-1" sends a heartbeat
    Then the Fleet view shows the agent "prod-1" as healthy

  Scenario: The Fleet view shows a failing agent, and healthy again after the fix
    Given the generator "team-robin" has applied a folder as the stack "payments"
    And the admin has sent the stack "payments" to the agent "prod-1"
    When the agent "prod-1" sends a heartbeat
    And the agent "prod-1" reports the stack "payments" as failing
    Then the Fleet view shows the agent "prod-1" as failing
    When the generator "team-robin" applies a changed folder as the stack "payments"
    And the agent "prod-1" reports the stack "payments" as healthy
    Then the Fleet view shows the agent "prod-1" as healthy

  # BROKKR-T-0393: a deleted stack left its failing and degraded rows in the
  # fleet health counts.
  Scenario: After a stack delete, the Fleet view shows no failing and no degraded pill
    Given the generator "team-robin" has applied a folder as the stack "payments"
    And the generator "team-robin" has applied a folder as the stack "ledger"
    And the admin has sent the stack "payments" to the agent "prod-1"
    And the admin has sent the stack "ledger" to the agent "prod-1"
    When the agent "prod-1" sends a heartbeat
    And the agent "prod-1" reports the stack "payments" as failing
    And the agent "prod-1" reports the stack "ledger" as degraded
    Then the Fleet view shows the agent "prod-1" as failing
    When the generator "team-robin" deletes the stack "payments"
    And the generator "team-robin" deletes the stack "ledger"
    Then the Fleet view shows no failing and no degraded pill for the agent "prod-1"

  Scenario: The Overview counts agree with the Fleet view
    Given the generator "team-robin" has applied a folder as the stack "payments"
    And the admin has sent the stack "payments" to the agent "prod-1"
    When the agent "prod-1" sends a heartbeat
    And the agent "prod-1" reports the stack "payments" as degraded
    Then the Fleet view shows the agent "prod-1" as degraded
    And the Overview counts agree with the Fleet view

  # BROKKR-T-0396: an agent that stopped talking read healthy, and the
  # Overview counted it as healthy and up. The console marks an agent with no
  # open connection and a heartbeat older than 60 seconds as offline.
  Scenario: A stopped agent reads offline, and the Overview does not count it up
    When the agent "prod-1" sends a heartbeat
    Then the Fleet view shows the agent "prod-1" as healthy
    And the Overview shows the cluster of the agent "prod-1" as 1 of 1 up
    When the agent "prod-1" stays silent for 65 seconds
    Then the Fleet view shows the agent "prod-1" as offline
    And the Overview shows the cluster of the agent "prod-1" as 0 of 1 up

  Scenario: The Overview shows a pending and a claimed work order
    Given the admin has created the work order "backup" for the agent "prod-1"
    And the admin has created the work order "restore" for the agent "prod-1"
    When the agent "prod-1" claims the work order "restore"
    Then the Overview shows the work order "backup" as pending
    And the Overview shows the work order "restore" as claimed
