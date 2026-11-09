Feature: Agent onboarding
  An admin creates an agent with its generators and gets its PAK and the Helm
  command once. The agent is inactive until the admin activates it. The
  agent's PAK is its identity: the agent finds its record by the PAK, and a
  configured name or cluster is a check.

  Scenario: The admin creates an agent and gets its PAK and the Helm command once
    Given the admin has created the generator "team-robin"
    When the admin creates the agent "prod-1" in cluster "cluster-a" for the generator "team-robin"
    Then the agent "prod-1" exists
    And the output shows the PAK once
    And the output shows the Helm command for the agent "prod-1" in cluster "cluster-a" with the generator "team-robin"
    And the output says the agent is inactive and how to activate it

  Scenario: A new agent is inactive until the admin activates it
    Given the admin has created the agent "prod-1" in cluster "cluster-a"
    Then the agent "prod-1" is INACTIVE
    When the admin activates the agent "prod-1"
    Then the agent "prod-1" is ACTIVE

  Scenario: A new agent is registered with its generator and with the system generator
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a" for the generator "team-robin"
    Then the agent "prod-1" is registered with the generator "team-robin"
    And the agent "prod-1" is registered with the system generator

  Scenario: The agent's PAK names its record
    Given the admin has created the agent "prod-1" in cluster "cluster-a"
    When the agent "prod-1" asks who it is
    Then the answer names the agent "prod-1" in cluster "cluster-a"

  Scenario: An agent with only its PAK finds its record
    Given the admin has created the agent "prod-1" in cluster "cluster-a"
    When the agent "prod-1" starts with its PAK and no name or cluster
    Then the agent finds the record "prod-1"

  Scenario: A wrong configured name is refused with the precise error
    Given the admin has created the agent "prod-1" in cluster "cluster-a"
    When the agent "prod-1" starts with its PAK and the name "prod-2"
    Then the agent stops
    And the error says the record has the name "prod-1" and the configuration says "prod-2"
    And the error names the Helm value "agentName"

  Scenario: A wrong configured cluster is refused with the precise error
    Given the admin has created the agent "prod-1" in cluster "cluster-a"
    When the agent "prod-1" starts with its PAK and the cluster "cluster-b"
    Then the agent stops
    And the error says the record has the cluster "cluster-a" and the configuration says "cluster-b"
    And the error names the Helm value "clusterName"
