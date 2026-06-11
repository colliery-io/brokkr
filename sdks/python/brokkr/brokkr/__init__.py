"""Ergonomic Python wrapper around brokkr-broker-client."""

from brokkr.client import ApplyResult, BrokkrClient
from brokkr.errors import BrokkrError

# Re-export the typed ErrorResponse model so consumers don't need to dig into
# the generated package layout.
from brokkr_broker_client.models import ErrorResponse

# The generated `Generator` model clashes with `typing.Generator` and produces
# mypy false positives when imported into PEP 604 unions. Re-export under a
# clearer name; the original is still reachable as
# `brokkr_broker_client.models.Generator`.
from brokkr_broker_client.models import Generator as TemplateGenerator

__all__ = ["ApplyResult", "BrokkrClient", "BrokkrError", "ErrorResponse", "TemplateGenerator"]
