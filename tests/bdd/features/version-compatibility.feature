Feature: Version compatibility of the agent and the broker
  An upgrade goes broker first and agents later, so for some time a released
  agent talks to a newer broker. The reverse also happens: a new agent in a
  cluster whose broker is not upgraded yet. These scenarios must hold in each
  pairing: both sides from this checkout, the agent of the last release with
  this broker, and this agent with the broker of the last release. They use
  the agent that runs in the cluster of the test stack.

  A scenario with the tag pairing-old-agent or pairing-old-broker runs in that
  pairing only. A scenario with needs-new-agent or needs-new-broker is left
  out when that side is the last release.

  Background:
    Given the admin has created the generator "team-robin"
    And the cluster agent is registered with the generator "team-robin" and active

  Scenario: The agent in the cluster is live in the fleet
    Then the fleet record of the cluster agent has a heartbeat under 60 seconds old
    And the fleet record of the cluster agent has an open connection

  Scenario: A stack sent to the agent deploys and its status names the agent
    Given the generator "team-robin" has applied a folder as the stack "payments"
    When the generator "team-robin" sends the stack "payments" to the cluster agent
    Then the status of the stack "payments" names the cluster agent within three minutes

  Scenario: The events and the log lines of a stack read back
    Given the generator "team-robin" has applied a folder with a pod that prints a line as the stack "ledger"
    When the generator "team-robin" sends the stack "ledger" to the cluster agent
    Then the events of the stack "ledger" name its pod within three minutes
    And the logs of the stack "ledger" have the line of its pod within three minutes
    And the broker has dropped no frame of an agent
    And the agent in the cluster has dropped no frame of the broker

  Scenario: A work order of an admin runs to its end
    When the admin launches the work order "migrate" on the cluster agent with a job that prints a line
    Then the log has the work order "migrate" as a success within three minutes
    And the broker has dropped no frame of an agent
    And the agent in the cluster has dropped no frame of the broker

  @needs-new-agent @needs-new-broker
  Scenario: The log line and the events of a work order read back
    When the admin launches the work order "migrate" on the cluster agent with a job that prints a line
    Then the log has the work order "migrate" as a success within three minutes
    And the logs of the work order "migrate" have the line of its job
    And the events of the work order "migrate" name its job

  # An agent of the last release does not send the log lines and the events
  # of a work order. The routes of the broker still answer.
  @pairing-old-agent
  Scenario: The work order of an old agent has no log lines and no events
    When the admin launches the work order "migrate" on the cluster agent with a job that prints a line
    Then the log has the work order "migrate" as a success within three minutes
    And the logs of the work order "migrate" are empty
    And the events of the work order "migrate" are empty

  # A broker of the last release cannot read a frame with a work order id and
  # no stack id. It gives no capability header, so the agent must send none.
  @pairing-old-broker
  Scenario: A new agent sends an old broker no log lines and no events of a work order
    When the admin launches the work order "migrate" on the cluster agent with a job that prints a line
    Then the log has the work order "migrate" as a success within three minutes
    And the broker has dropped no frame of an agent
