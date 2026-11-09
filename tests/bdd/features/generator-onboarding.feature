Feature: Generator onboarding
  An admin creates a generator and gets its PAK once. The admin registers the
  generator's agents. After that the generator works with its own PAK.

  Scenario: The admin creates a generator and gets its PAK once
    When the admin creates the generator "team-robin"
    Then the generator "team-robin" exists
    And the output shows the PAK once
    And the output names the next step

  Scenario: The admin registers an agent with the generator and the generator lists it
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a"
    When the admin registers the agent "prod-1" with the generator "team-robin"
    Then the agent "prod-1" is registered with the generator "team-robin"
    And the generator "team-robin" lists the agent "prod-1"

  Scenario: Registering an agent again changes nothing
    Given the admin has created the generator "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a"
    And the admin has registered the agent "prod-1" with the generator "team-robin"
    When the admin registers the agent "prod-1" with the generator "team-robin"
    Then the command says it changed nothing

  Scenario: A repeat name is refused
    Given the admin has created the generator "team-robin"
    When the admin creates the generator "team-robin" again
    Then the command is refused
    And there is one generator "team-robin"

  Scenario: The generator list shows the generator and not the system generator
    Given the admin has created the generator "team-robin"
    When the admin lists the generators
    Then the list has the generator "team-robin"
    And the list has no system generator

  Scenario: The generator's PAK names its own record
    Given the admin has created the generator "team-robin"
    When the generator "team-robin" asks who it is
    Then the answer names the generator "team-robin"

  Scenario: The generator's PAK reads nothing else
    Given the admin has created the generator "team-robin"
    And the admin has created the generator "team-sam"
    When the generator "team-robin" reads the record of the generator "team-sam"
    Then the broker refuses
    When the generator "team-robin" lists the generators
    Then the command is refused
