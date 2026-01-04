---
id: work-order-failure-messages-not
level: task
title: "Work order failure messages not stored during retry cycles"
short_code: "BROKKR-T-0047"
created_at: 2025-12-22T02:39:54.259681+00:00
updated_at: 2025-12-29T01:17:22.533215+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Work order failure messages not stored during retry cycles

## Objective

When a work order fails and enters `RETRY_PENDING` status, the error message from the agent is discarded. Operators have no visibility into why work orders are failing until they exhaust all retries and move to `work_order_log`.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All operators using work orders
- **Reproduction Steps**: 
  1. Create a work order with invalid YAML or targeting a non-existent resource
  2. Agent claims and attempts to execute, fails
  3. Work order enters `RETRY_PENDING` status
  4. Check the work order via API or UI - no error information available
- **Expected vs Actual**: 
  - **Expected**: Error message should be visible showing why the work order failed
  - **Actual**: No error information stored; only `retry_count` increments

### Root Cause

The `work_orders` table lacks columns for storing error information:
- No `last_error` column for the most recent failure message
- No `last_error_at` column for when the error occurred
- The `complete_failure` DAL function discards the error message when scheduling retries

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `work_orders` table has `last_error` (TEXT) and `last_error_at` (TIMESTAMPTZ) columns
- [ ] When agent reports failure, error message is stored even if retrying
- [ ] API returns `last_error` and `last_error_at` in work order responses
- [ ] UI displays error message for work orders in `RETRY_PENDING` status
- [ ] Error is cleared when work order returns to `PENDING` for retry

## Implementation Notes

### Technical Approach

1. **Database Migration**: Add columns to `work_orders` table
   ```sql
   ALTER TABLE work_orders ADD COLUMN last_error TEXT;
   ALTER TABLE work_orders ADD COLUMN last_error_at TIMESTAMPTZ;
   ```

2. **Model Update**: Add fields to `WorkOrder` struct in `brokkr-models`

3. **DAL Update**: Modify `complete_failure` in `work_orders.rs` to store error:
   ```rust
   diesel::update(work_orders::table.filter(work_orders::id.eq(work_order_id)))
       .set((
           work_orders::status.eq(WORK_ORDER_STATUS_RETRY_PENDING),
           work_orders::retry_count.eq(new_retry_count),
           work_orders::next_retry_after.eq(next_retry),
           work_orders::last_error.eq(&error_message),  // NEW
           work_orders::last_error_at.eq(Utc::now()),   // NEW
           // ...
       ))
   ```

4. **UI Update**: Display `last_error` in Jobs panel for RETRY_PENDING work orders

### Files to Modify
- `migrations/YYYYMMDD_add_work_order_error_columns.sql`
- `crates/brokkr-models/src/models/work_orders.rs`
- `crates/brokkr-models/src/schema.rs`
- `crates/brokkr-broker/src/dal/work_orders.rs`
- `examples/ui-slim/src/App.js`

## Status Updates

*To be added during implementation*