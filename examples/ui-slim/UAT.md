# Brokkr UI Slim — UAT Script

A focused checklist for validating that the ui-slim demo still works end to end.
Each step lists the **action**, the **input values** to use, and the
**expected behavior** to confirm. Tick boxes as you go.

This is intentionally narrower than `DEMO_WALKTHROUGH.md`. The goal here is
**regression detection**, not pedagogy. Run this after any change that touches
the broker API, the SDK, or the demo's `api.js` / components.

---

## Pre-flight

| # | Action | Input / Where | Expected |
|---|---|---|---|
| P1 | Bring up the stack | `angreal local up` (terminal) | All containers come up. `broker-1`, `agent-1`, `brokkr-ui-1`, `webhook-catcher-1` show `Healthy`/`Up` in `docker ps`. |
| P2 | Open the UI | http://localhost:3001 | Page loads. No "Failed to fetch" banner. Header reads "Brokkr". |
| P3 | Open browser DevTools → Network | F12, Network tab | Every XHR/fetch to `localhost:3000/api/v1/...` carries `Authorization: Bearer brokkr_BR3rVsDa_…`. (Confirms SDK token injection works.) |
| P4 | Open browser DevTools → Console | F12, Console tab | No red errors on page load. Yellow deprecation warnings from React dev-server are fine. |

**Stop here if any P-step fails.** Everything below assumes the stack is reachable.

---

## 1. Agents

| # | Action | Input | Expected |
|---|---|---|---|
| A1 | Click **Agents** tab | — | List renders with at least one agent (the integration test agent). |
| A2 | Note any pre-existing agent | — | Columns show: name, cluster, status, labels count, target count, last heartbeat. Heartbeat is within the last 60s. |
| A3 | Click into an agent row | — | Detail panel/modal opens. Sections visible: Labels, Annotations, Targets, Events. |
| A4 | Activate an INACTIVE agent (if present) | Click **Activate** | Status flips to `ACTIVE` (green). No error toast. Network: `PUT /api/v1/agents/{id}` returns 200. |
| A5 | Add a label | Type `uat-smoke` → Enter (or +) | Label appears in the list. Network: `POST /api/v1/agents/{id}/labels` returns 200. |
| A6 | Remove that label | Click × on `uat-smoke` | Label disappears. Network: `DELETE /api/v1/agents/{id}/labels/uat-smoke` returns 204. |
| A7 | Add an annotation | key=`uat`, value=`true` | Annotation row appears. Network: `POST /api/v1/agents/{id}/annotations` returns 200. |
| A8 | Remove that annotation | Click × on `uat` | Annotation row disappears. Network: `DELETE /api/v1/agents/{id}/annotations/uat` returns 204. |
| A9 | View events | Scroll Events section | Either empty state ("no events yet") or a list of recent events with type/status/timestamp. No spinner stuck. |
| A10 | Create a new agent | Name=`uat-agent`, cluster=`uat-cluster`, click **Create** | New agent appears with an initial PAK shown once. Status `INACTIVE`. Network: `POST /api/v1/agents` returns 200 with `{ agent, initial_pak }`. |
| A11 | Rotate the new agent's PAK | Click **Rotate PAK** in detail | A new PAK is shown. Old one immediately invalid. Network: `POST /api/v1/agents/{id}/rotate-pak` returns 200. |
| A12 | Delete the new agent (cleanup) | Click **Delete** (if exposed; else skip) | Agent removed from list. Network: `DELETE /api/v1/agents/{id}` returns 204. |

---

## 2. Generators

| # | Action | Input | Expected |
|---|---|---|---|
| G1 | Click **Generators** tab | — | List renders. Existing generators visible (e.g. an integration generator). |
| G2 | Create a generator | name=`uat-generator`, description=`UAT test` | Generator appears with an initial PAK shown once. Network: `POST /api/v1/generators` returns 200. |
| G3 | Rotate generator PAK | Click **Rotate PAK** on `uat-generator` | New PAK shown. Network: `POST /api/v1/generators/{id}/rotate-pak` returns 200. |
| G4 | Delete the generator (cleanup) | Click **Delete** | Generator removed. Network: `DELETE /api/v1/generators/{id}` returns 204. |

---

## 3. Stacks

| # | Action | Input | Expected |
|---|---|---|---|
| S1 | Click **Stacks** tab | — | List renders. Create form visible. |
| S2 | Create a stack | name=`uat-stack`, description=`UAT smoke`, generator=any | Stack appears. Network: `POST /api/v1/stacks` returns 200. |
| S3 | Open the stack | Click `uat-stack` | Detail panel opens with Labels, Annotations, Deployment Objects sections. |
| S4 | Add a stack label | `uat` | Label appears. Network: `POST /api/v1/stacks/{id}/labels` returns 200. **Note:** sends a plain JSON string body, not an object. Confirm this works. |
| S5 | Add a stack annotation | key=`env`, value=`uat` | Annotation row appears. Network: `POST /api/v1/stacks/{id}/annotations` returns 200. |
| S6 | Create a deployment object | Paste a minimal NS YAML (below*), uncheck "deletion marker" | Deployment row appears with a sequence number. Network: `POST /api/v1/stacks/{id}/deployment-objects` returns 200. |
| S7 | View the stack's health | Open Stack Health section/modal | Health view renders. May be "unknown" if agent hasn't reported yet — that's fine. Network: `GET /api/v1/stacks/{id}/health` returns 200. |

**Minimal NS YAML for S6:**

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: uat-test-ns
```

---

## 4. Templates

| # | Action | Input | Expected |
|---|---|---|---|
| T1 | Click **Templates** tab | — | List renders. |
| T2 | Create a template | name=`uat-template`, content=Tera YAML below*, schema=below* | Template appears. Network: `POST /api/v1/templates` returns 200. |
| T3 | Add a template label | `uat` | Label appears. Network: `POST /api/v1/templates/{id}/labels` returns 200. |
| T4 | Instantiate the template against `uat-stack` | template=`uat-template`, stack=`uat-stack`, params=`{"name":"hello"}` | New deployment object created on the stack. Network: `POST /api/v1/stacks/{stack_id}/deployment-objects/from-template` returns 200. |
| T5 | Update the template | Change description to "UAT updated" | Updated. Network: `PUT /api/v1/templates/{id}` returns 200. |
| T6 | Delete the template (cleanup) | Click **Delete** | Removed. Network: `DELETE /api/v1/templates/{id}` returns 204. |

**T2 inputs:**

Template content (Tera):
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ name }}
  namespace: uat-test-ns
data:
  message: "hello from {{ name }}"
```

Parameters schema (JSON Schema):
```json
{
  "type": "object",
  "properties": { "name": { "type": "string" } },
  "required": ["name"]
}
```

---

## 5. Work Orders (Jobs)

| # | Action | Input | Expected |
|---|---|---|---|
| W1 | Click **Jobs** / **Work Orders** tab | — | List renders. Existing work orders visible (if any). |
| W2 | Filter by status | Pick `PENDING` | List narrows to PENDING. Network: `GET /api/v1/work-orders?status=PENDING` returns 200. |
| W3 | Filter by work type | Pick `build` (or any) | List filters. Network: query string includes `work_type=`. |
| W4 | Create a build work order | Click "Create build work order" (or equivalent) | Work order appears with status `PENDING`. Network: `POST /api/v1/work-orders` returns 200. |
| W5 | View work order detail | Click into the new WO | Detail shows targeting, retry config, status, timestamps. Network: `GET /api/v1/work-orders/{id}` returns 200. |
| W6 | View work order log | Scroll to log section | List of completed work orders renders. Network: `GET /api/v1/work-order-log` returns 200. |
| W7 | Delete the new work order (cleanup) | Click **Delete** | Removed. Network: `DELETE /api/v1/work-orders/{id}` returns 204. |

---

## 6. Diagnostics

| # | Action | Input | Expected |
|---|---|---|---|
| D1 | Open a deployment object you created in S6 | — | Detail shows YAML, status, health. |
| D2 | Request a diagnostic | agent=any active agent, retention=60 | Diagnostic request created. Network: `POST /api/v1/deployment-objects/{id}/diagnostics` returns 200. |
| D3 | Open the diagnostic | Click into the new diagnostic | Status shows `PENDING`, `CLAIMED`, or `COMPLETED` depending on agent activity. Network: `GET /api/v1/diagnostics/{id}` returns 200. |
| D4 | (If COMPLETED) View result | Scroll to result section | Pod statuses, events, log tails render. No "undefined" or "[object Object]" strings. |

---

## 7. Health

| # | Action | Input | Expected |
|---|---|---|---|
| H1 | Open deployment object from S6 | — | Health section visible. Status may be "unknown" until agent reports. |
| H2 | View deployment health detail | Click **Health** | Per-agent health records list. Overall status computed. Network: `GET /api/v1/deployment-objects/{id}/health` returns 200. |
| H3 | View stack-level health | Stack detail → Health | Per-deployment-object summary renders. Network: `GET /api/v1/stacks/{id}/health` returns 200. |

---

## 8. Webhooks

| # | Action | Input | Expected |
|---|---|---|---|
| WH1 | Click **Webhooks** tab | — | List renders. May be empty. |
| WH2 | View available event types | Open the dropdown / list | List includes `deployment.applied`, `deployment.failed`, etc. Network: `GET /api/v1/webhooks/event-types` returns 200. |
| WH3 | Create a webhook → webhook-catcher | name=`uat-webhook`, url=`http://webhook-catcher:8080/webhook`, events=[`deployment.applied`] | Webhook created and shown in list. Network: `POST /api/v1/webhooks` returns 200. |
| WH4 | Open the webhook | Click row | Detail shows fields, target labels, retry config. `has_url`=true, `has_auth_header`=false. Network: `GET /api/v1/webhooks/{id}` returns 200. |
| WH5 | Test the webhook | Click **Test** (if exposed) | Network: `POST /api/v1/webhooks/{id}/test` returns 200. Webhook-catcher receives the test payload (see WH8). |
| WH6 | Trigger a real event | Re-apply a deployment object (S6 again, slightly different YAML) | Agent applies → broker emits `deployment.applied` → webhook fires. |
| WH7 | View deliveries | Webhook detail → Deliveries | Recent deliveries list with status (`pending`, `success`, `failed`). Network: `GET /api/v1/webhooks/{id}/deliveries` returns 200. |
| WH8 | Open webhook-catcher | http://localhost:8090/messages (or the UI's webhook-catcher panel) | Recent messages include the events from WH5 / WH6. |
| WH9 | Delete the webhook (cleanup) | Click **Delete** | Removed. Network: `DELETE /api/v1/webhooks/{id}` returns 204. |

---

## 9. Admin / Metrics

| # | Action | Input | Expected |
|---|---|---|---|
| M1 | Open **Admin** or **Metrics** panel | — | Renders a list of Prometheus metrics with values. |
| M2 | Verify v1 metrics | Look for `brokkr_active_agents`, `brokkr_heartbeat_sent_total`, etc. | Values are present and non-NaN. Network: `GET /metrics` returns text/plain. (This endpoint is intentionally NOT through the SDK — it's not v1.) |

---

## 10. Cleanup

| # | Action | Input | Expected |
|---|---|---|---|
| C1 | Use the demo's "Cleanup" panel if exposed | — | Removes test resources, deactivates demo agents, clears webhook catcher. |
| C2 | Or manually: delete any `uat-*` resources you created above | — | All `uat-*` agents/generators/stacks/templates/webhooks gone. |
| C3 | Tear down the stack | `angreal local down --hard` (terminal) | All containers stop; volumes removed. |

---

## SDK regression sentinels

These are the specific things the SDK migration could have broken. If any of these
behaviors differ from the pre-migration baseline, file it as a bug against
`@brokkr/client` (T-C4) or the spec (Phase A) rather than working around it in
the demo.

| Sentinel | What to check | Why |
|---|---|---|
| Path-with-id substitution | Network requests for `/agents/{id}/labels` show the real UUID, not the literal `{id}` | If `params.path` plumbing breaks in openapi-fetch, you'll see `{id}` in the URL. |
| Path-with-two-params | `DELETE /agents/{id}/labels/{label}` URL has both values substituted | Two-segment path substitution is a separate code path in openapi-fetch. |
| String-body endpoints | `POST /stacks/{id}/labels` body is `"my-label"` (a JSON string), not `{ label: "..." }` | This was the route Python's `openapi-python-client` *skipped*. openapi-fetch handles it; verify the byte-level body is right. |
| Query params | `GET /work-orders?status=PENDING` sends `status` as a query param, not a path segment | `params.query` plumbing. |
| Authorization header | Every v1 request carries `Authorization: Bearer brokkr_...` | If you see an unauthenticated 401, header injection failed. |
| 204 No Content | DELETE responses don't throw "unexpected response" errors | The wrapper's `unwrap()` returns `null` on 204. |
| Typed error → toast | A deliberate 404 (e.g. open a deleted agent's detail) shows the broker's `message`, not "[object Object]" | The `unwrap()` formats `{ code, message }` into a readable string. |
| No `{ data, error, response }` leakage | Components see `await api.foo()` returning the success body directly, not the tuple | If a component logs `{data, error, response}`, the `unwrap()` was bypassed somewhere. |

---

## Reporting

If anything fails, copy:
1. The step number (e.g. **WH5 fail**).
2. The browser Network entry (status code + response body).
3. The browser Console entry (red error, if any).
4. `docker logs brokkr-dev-broker-1 --tail 50` for the relevant timeframe.

That's enough to root-cause without re-running the whole script.
