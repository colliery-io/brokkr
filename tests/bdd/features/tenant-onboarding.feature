Feature: Tenant onboarding
  An admin creates a tenant and gets its PAK once. The admin registers the
  tenant's agents. After that the tenant works with its own PAK.

  Scenario: The admin creates a tenant and gets its PAK once
    When the admin creates the tenant "team-robin"
    Then the tenant "team-robin" exists
    And the output shows the PAK once
    And the output names the next step

  Scenario: The admin registers an agent with the tenant and the tenant lists it
    Given the admin has created the tenant "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a"
    When the admin registers the agent "prod-1" with the tenant "team-robin"
    Then the agent "prod-1" is registered with the tenant "team-robin"
    And the tenant "team-robin" lists the agent "prod-1"

  Scenario: Registering an agent again changes nothing
    Given the admin has created the tenant "team-robin"
    And the admin has created the agent "prod-1" in cluster "cluster-a"
    And the admin has registered the agent "prod-1" with the tenant "team-robin"
    When the admin registers the agent "prod-1" with the tenant "team-robin"
    Then the command says it changed nothing

  Scenario: A repeat name is refused
    Given the admin has created the tenant "team-robin"
    When the admin creates the tenant "team-robin" again
    Then the command is refused
    And there is one tenant "team-robin"

  Scenario: The tenant list shows the tenant and not the system generator
    Given the admin has created the tenant "team-robin"
    When the admin lists the tenants
    Then the list has the tenant "team-robin"
    And the list has no system generator

  Scenario: The tenant's PAK names its own record
    Given the admin has created the tenant "team-robin"
    When the tenant "team-robin" asks who it is
    Then the answer names the tenant "team-robin"

  Scenario: The tenant's PAK reads nothing else
    Given the admin has created the tenant "team-robin"
    And the admin has created the tenant "team-sam"
    When the tenant "team-robin" reads the record of the tenant "team-sam"
    Then the broker refuses
    When the tenant "team-robin" lists the tenants
    Then the command is refused
