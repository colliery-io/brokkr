Feature: The rules that keep generators apart
  A generator works only inside its own scope. It cannot create generators or
  agents, cannot register agents, and cannot read another generator's stack.
  An agent receives a generator's stacks only once it is registered with that
  generator, and the admin cannot skip that step. The same holds for work
  orders: a generator reads and deletes only its own, and it reaches only the
  agents registered with it.

  Background:
    Given the admin has created the generator "team-robin"
    And the admin has created the generator "team-sam"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the generator "team-robin"
    And the admin has created the agent "stray-1" in cluster "cluster-a"

  Scenario: A generator cannot register agents
    When the generator "team-robin" registers the agent "stray-1" with itself
    Then the command is refused
    And the agent "stray-1" is not registered with the generator "team-robin"

  Scenario: A generator cannot create agents
    When the generator "team-robin" creates the agent "edge-1" in cluster "cluster-a"
    Then the command is refused
    And there is no agent "edge-1"

  Scenario: A generator cannot create generators
    When the generator "team-robin" creates the generator "team-jamie"
    Then the command is refused
    And there is no generator "team-jamie"

  Scenario: A generator cannot read another generator's stack
    Given the generator "team-sam" has applied a folder as the stack "ledger"
    When the generator "team-robin" asks for the status of the stack "ledger"
    Then the command is refused
    When the generator "team-robin" reads the health of the stack "ledger" by its id
    Then the broker refuses

  Scenario: A generator sees only its own stacks and agents
    Given the generator "team-sam" has applied a folder as the stack "ledger"
    And the generator "team-robin" has applied a folder as the stack "payments"
    When the generator "team-robin" lists its stacks
    Then the list has the stack "payments"
    And the list does not have the stack "ledger"
    When the generator "team-robin" lists its agents
    Then the list has the agent "prod-1"
    And the list does not have the agent "stray-1"

  Scenario: A target for an unregistered agent is refused, even for the admin
    Given the generator "team-robin" has applied a folder as the stack "payments"
    When the admin sends the stack "payments" to the agent "stray-1"
    Then the command is refused
    And the error says the agent "stray-1" is not registered with the generator that owns the stack "payments"
    And the error names the command that registers it

  Scenario: A generator cannot reach an agent that is not registered with it
    Given the generator "team-robin" has applied a folder as the stack "payments"
    When the generator "team-robin" sends the stack "payments" to the agent "stray-1" by its id
    Then the command is refused

  Scenario: A generator cannot read the labels of an agent that is not registered with it
    Given the admin has labelled the agent "stray-1" with "env:bdd"
    When the generator "team-robin" reads the labels of the agent "stray-1"
    Then the broker refuses

  Scenario: A registered agent receives the generator's stack
    Given the generator "team-robin" has applied a folder as the stack "payments"
    When the generator "team-robin" sends the stack "payments" to the agent "prod-1"
    Then the command says it sent the stack "payments" to the agent "prod-1"

  Scenario: A generator cannot read or delete another generator's work order
    Given the generator "team-robin" has created the work order "backup" for the agent "prod-1"
    When the generator "team-sam" asks for the status of the work order "backup"
    Then the command is refused
    And the error says the work order belongs to another owner
    When the generator "team-sam" reads the logs of the work order "backup"
    Then the command is refused
    And the error says the work order belongs to another owner
    When the generator "team-sam" reads the events of the work order "backup"
    Then the command is refused
    And the error says the work order belongs to another owner
    When the generator "team-sam" deletes the work order "backup"
    Then the command is refused
    And the error says the work order belongs to another owner
    And the work order "backup" is PENDING

  Scenario: A generator does not see another generator's work order in its list
    Given the generator "team-robin" has created the work order "backup" for the agent "prod-1"
    When the generator "team-sam" lists its work orders
    Then the command says no work orders wait or run

  Scenario: A generator cannot read the result of another generator's work order
    Given the generator "team-robin" has created the work order "backup" for the agent "prod-1"
    When the agent "prod-1" claims the work order "backup"
    And the agent "prod-1" completes the work order "backup"
    And the generator "team-sam" reads the history of its work orders
    Then the history does not have the work order "backup"
    When the generator "team-sam" asks for the status of the work order "backup"
    Then the command is refused
    And the error says the work order belongs to another owner
    When the generator "team-robin" reads the history of its work orders
    Then the history has the work order "backup" as a success

  Scenario: A work order for an agent that is not registered with the generator is refused
    When the generator "team-robin" creates the work order "backup" for the agent "stray-1" by its id
    Then the command is refused
    And the error says the agent is not registered with the generator
    And the error names the command that registers it
    When the generator "team-robin" lists its work orders
    Then the command says no work orders wait or run
