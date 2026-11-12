Feature: Agent lifecycle
  After onboarding, an agent sends heartbeats and reports the health of what
  it serves. An admin pauses it, takes a stack from it, or deletes it. The
  fleet record and the stack health say what is true after each step. The
  scenario plays the agent with the agent's PAK.

  Background:
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the generator "team-robin"

  Scenario: An active agent that sends a heartbeat has a fresh heartbeat in the fleet
    Given the admin has activated the agent "prod-1"
    When the agent "prod-1" sends a heartbeat
    Then the fleet record of the agent "prod-1" is ACTIVE
    And the fleet record of the agent "prod-1" has a heartbeat under 30 seconds old

  Scenario: An agent that goes silent has a heartbeat that ages
    Given the admin has activated the agent "prod-1"
    When the agent "prod-1" sends a heartbeat
    And the agent "prod-1" stays silent for 5 seconds
    Then the fleet record of the agent "prod-1" has a heartbeat at least 5 seconds old
    And the fleet record of the agent "prod-1" has no open connection

  Scenario: A paused agent stays in the fleet and reads inactive
    Given the admin has activated the agent "prod-1"
    When the admin pauses the agent "prod-1"
    Then the agent "prod-1" is INACTIVE
    And the fleet record of the agent "prod-1" is INACTIVE

  Scenario: An agent that loses a stack no longer counts its health
    Given the admin has activated the agent "prod-1"
    And the generator "team-robin" has applied a folder as the stack "payments"
    And the admin has sent the stack "payments" to the agent "prod-1"
    And the agent "prod-1" has reported the stack "payments" as failing
    Then the fleet record of the agent "prod-1" counts 1 failing record
    And the health of the stack "payments" lists the agent "prod-1"
    When the admin takes the stack "payments" from the agent "prod-1"
    Then the fleet record of the agent "prod-1" counts 0 failing records
    And the health of the stack "payments" does not list the agent "prod-1"

  Scenario: A deleted agent leaves the fleet and its PAK stops
    When the admin deletes the agent "prod-1"
    Then the fleet does not list the agent "prod-1"
    And the PAK of the agent "prod-1" is refused

  # BROKKR-T-0394: the registrations list of a generator has no join to the
  # agents, so it keeps the registration of a deleted agent.
  @known-bug
  Scenario: A deleted agent is not in the registrations of its generator
    When the admin deletes the agent "prod-1"
    Then the registrations of the generator "team-robin" do not name the agent "prod-1"

  Scenario: An agent that serves a stack by a label alone reads its deployment object
    Given the admin has labelled the agent "prod-1" with "env:lifecycle"
    And the generator "team-robin" has applied a folder as the stack "payments"
    And the generator "team-robin" has labelled the stack "payments" with "env:lifecycle"
    Then the target state of the agent "prod-1" has the stack "payments"
    When the agent "prod-1" reads the deployment object of the stack "payments" by its id
    Then the broker answers
