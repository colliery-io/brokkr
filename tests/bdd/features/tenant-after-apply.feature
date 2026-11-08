Feature: The tenant after apply
  With its own PAK, a tenant applies a folder of manifests, sees where its
  stack went, sees the labels of its registered agents, rotates its PAK and
  deletes its stack. No admin after the hand-off.

  Background:
    Given the admin has created the tenant "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the tenant "team-robin"

  Scenario: The tenant applies a folder and gets a stack
    When the tenant "team-robin" applies a folder as the stack "payments"
    Then the command says it created the stack "payments" with a first revision
    And the tenant "team-robin" lists the stack "payments"

  Scenario: Applying the same folder again changes nothing
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" applies the same folder as the stack "payments"
    Then the command says the stack "payments" is unchanged

  Scenario: The tenant sees the status of its stack before any agent reports
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" asks for the status of the stack "payments"
    Then the status shows the latest revision and the health
    And the status says no agent has reported yet

  Scenario: The tenant reads the events and the logs of its stack
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" reads the events of the stack "payments"
    Then the command succeeds
    When the tenant "team-robin" reads the logs of the stack "payments"
    Then the command succeeds

  Scenario: The tenant sees the labels of its registered agents
    Given the admin has labelled the agent "prod-1" with "env:bdd"
    When the tenant "team-robin" lists its agents
    Then the list shows the agent "prod-1" with the label "env:bdd"

  Scenario: A stack label says which registered agents it reaches
    Given the admin has labelled the agent "prod-1" with "env:bdd"
    And the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" labels the stack "payments" with "env:bdd"
    Then the command says the labels of the stack "payments" match the agent "prod-1"

  Scenario: A stack label that reaches no agent says so
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" labels the stack "payments" with "env:nowhere"
    Then the command says no agent matches the labels of the stack "payments"

  Scenario: The tenant rotates its PAK and the old one stops
    When the tenant "team-robin" rotates its PAK
    Then the output shows the PAK once and says the previous one stopped
    And the previous PAK of the tenant "team-robin" is refused
    And the new PAK of the tenant "team-robin" names the tenant

  Scenario: The tenant deletes its stack
    Given the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" deletes the stack "payments"
    Then the command says each agent removes the stack's resources on its next poll
    And the tenant "team-robin" no longer finds the stack "payments"

  Scenario: The tenant sees where its stack went
    Given the cluster agent is registered with the tenant "team-robin" and active
    And the tenant "team-robin" has applied a folder as the stack "payments"
    When the tenant "team-robin" sends the stack "payments" to the cluster agent
    Then the status of the stack "payments" names the cluster agent within three minutes
