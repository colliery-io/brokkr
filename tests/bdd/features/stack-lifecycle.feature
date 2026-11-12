Feature: Stack lifecycle
  After its first apply, a stack gets new revisions, fails, is fixed and is
  deleted. The stack health and the fleet record of its agent say what is
  true after each step. The scenario plays the agent with the agent's PAK.

  Background:
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the generator "team-robin"
    And the admin has activated the agent "prod-1"
    And the generator "team-robin" has applied a folder as the stack "payments"
    And the admin has sent the stack "payments" to the agent "prod-1"

  Scenario: A changed folder gives the stack a new revision, and the agent serves it
    When the generator "team-robin" applies a changed folder as the stack "payments"
    Then the command succeeds
    And the stack "payments" has 2 revisions
    And the agent "prod-1" serves the latest revision of the stack "payments"

  Scenario: A stack whose revision fails reads failing
    When the agent "prod-1" reports the stack "payments" as failing
    Then the health of the stack "payments" is failing
    And the fleet record of the agent "prod-1" counts 1 failing record

  Scenario: A new revision that fixes the stack makes it read healthy
    Given the agent "prod-1" has reported the stack "payments" as failing
    When the generator "team-robin" applies a changed folder as the stack "payments"
    And the agent "prod-1" reports the stack "payments" as healthy
    Then the health of the stack "payments" is healthy
    And the fleet record of the agent "prod-1" counts 0 failing records

  Scenario: A deleted stack leaves no failing and no degraded record
    Given the generator "team-robin" has applied a folder as the stack "ledger"
    And the admin has sent the stack "ledger" to the agent "prod-1"
    And the agent "prod-1" has reported the stack "payments" as failing
    And the agent "prod-1" has reported the stack "ledger" as degraded
    When the generator "team-robin" deletes the stack "payments"
    And the generator "team-robin" deletes the stack "ledger"
    Then the fleet record of the agent "prod-1" counts 0 failing records
    And the fleet record of the agent "prod-1" counts 0 degraded records
    And the deleted stack "payments" has no health

  # BROKKR-T-0394: a stack delete is soft and leaves the rows of
  # agent_targets, and the list of an agent's targets does not filter them.
  @known-bug
  Scenario: A deleted stack is not in the targets of its agent
    When the generator "team-robin" deletes the stack "payments"
    Then the targets of the agent "prod-1" do not name the deleted stack "payments"

  # BROKKR-T-0394: the health of one deployment object is read with no check
  # that the object is live, so the records of a deleted stack come back.
  @known-bug
  Scenario: The health of a revision of a deleted stack is not returned
    Given the agent "prod-1" has reported the stack "payments" as failing
    When the generator "team-robin" deletes the stack "payments"
    Then the last revision of the deleted stack "payments" has no health records

  # BROKKR-T-0394: a health report is stored for any agent and object, so a
  # late report after the delete writes the record again.
  @known-bug
  Scenario: A late health report on a deleted stack is not kept
    Given the agent "prod-1" has reported the stack "payments" as failing
    When the generator "team-robin" deletes the stack "payments"
    And the agent "prod-1" reports the last revision of the deleted stack "payments" as degraded
    Then the last revision of the deleted stack "payments" has no degraded record
