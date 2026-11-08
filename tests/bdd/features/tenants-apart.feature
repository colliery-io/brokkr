Feature: The rules that keep tenants apart
  A tenant works only inside its own scope. It cannot create tenants or
  agents, cannot register agents, and cannot read another tenant's stack.
  An agent receives a tenant's stacks only once it is registered with that
  tenant, and the admin cannot skip that step.

  Background:
    Given the admin has created the tenant "team-robin"
    And the admin has created the tenant "team-sam"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the tenant "team-robin"
    And the admin has created the agent "stray-1" in cluster "cluster-a"

  Scenario: A tenant cannot register agents
    When the tenant "team-robin" registers the agent "stray-1" with itself
    Then the command is refused
    And the agent "stray-1" is not registered with the tenant "team-robin"

  Scenario: A tenant cannot create agents
    When the tenant "team-robin" creates the agent "edge-1" in cluster "cluster-a"
    Then the command is refused
    And there is no agent "edge-1"

  Scenario: A tenant cannot create tenants
    When the tenant "team-robin" creates the tenant "team-jamie"
    Then the command is refused
    And there is no tenant "team-jamie"

  Scenario: A tenant cannot read another tenant's stack
    Given the tenant "team-sam" has applied a folder as the stack "ledger"
    When the tenant "team-robin" asks for the status of the stack "ledger"
    Then the command is refused
    When the tenant "team-robin" reads the health of the stack "ledger" by its id
    Then the broker refuses

  Scenario: A tenant sees only its own stacks and agents
    Given the tenant "team-sam" has applied a folder as the stack "ledger"
    And the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" lists its stacks
    Then the list has the stack "payments"
    And the list does not have the stack "ledger"
    When the tenant "team-robin" lists its agents
    Then the list has the agent "prod-1"
    And the list does not have the agent "stray-1"

  Scenario: A target for an unregistered agent is refused, even for the admin
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the admin sends the stack "payments" to the agent "stray-1"
    Then the command is refused
    And the error says the agent "stray-1" is not registered with the tenant that owns the stack "payments"
    And the error names the command that registers it

  Scenario: A tenant cannot reach an agent that is not registered with it
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" sends the stack "payments" to the agent "stray-1" by its id
    Then the command is refused

  Scenario: A tenant cannot read the labels of an agent that is not registered with it
    Given the admin has labelled the agent "stray-1" with "env:bdd"
    When the tenant "team-robin" reads the labels of the agent "stray-1"
    Then the broker refuses

  Scenario: A registered agent receives the tenant's stack
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" sends the stack "payments" to the agent "prod-1"
    Then the command says it sent the stack "payments" to the agent "prod-1"
