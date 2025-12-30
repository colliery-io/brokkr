/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

// @generated automatically by Diesel CLI.

diesel::table! {
    admin_role (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        pak_hash -> Text,
    }
}

diesel::table! {
    agent_annotations (id) {
        id -> Uuid,
        agent_id -> Uuid,
        #[max_length = 64]
        key -> Varchar,
        #[max_length = 64]
        value -> Varchar,
    }
}

diesel::table! {
    agent_events (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        agent_id -> Uuid,
        deployment_object_id -> Uuid,
        #[max_length = 50]
        event_type -> Varchar,
        #[max_length = 10]
        status -> Varchar,
        message -> Nullable<Text>,
    }
}

diesel::table! {
    agent_labels (id) {
        id -> Uuid,
        agent_id -> Uuid,
        #[max_length = 64]
        label -> Varchar,
    }
}

diesel::table! {
    agent_targets (id) {
        id -> Uuid,
        agent_id -> Uuid,
        stack_id -> Uuid,
    }
}

diesel::table! {
    agents (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        cluster_name -> Varchar,
        last_heartbeat -> Nullable<Timestamptz>,
        #[max_length = 50]
        status -> Varchar,
        pak_hash -> Text,
    }
}

diesel::table! {
    app_initialization (id) {
        id -> Int4,
        initialized_at -> Timestamptz,
    }
}

diesel::table! {
    deployment_health (id) {
        id -> Uuid,
        agent_id -> Uuid,
        deployment_object_id -> Uuid,
        #[max_length = 20]
        status -> Varchar,
        summary -> Nullable<Text>,
        checked_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    deployment_objects (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        sequence_id -> Int8,
        stack_id -> Uuid,
        yaml_content -> Text,
        yaml_checksum -> Text,
        submitted_at -> Timestamptz,
        is_deletion_marker -> Bool,
    }
}

diesel::table! {
    generators (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        pak_hash -> Nullable<Text>,
        last_active_at -> Nullable<Timestamptz>,
        is_active -> Bool,
    }
}

diesel::table! {
    stack_annotations (id) {
        id -> Uuid,
        stack_id -> Uuid,
        #[max_length = 64]
        key -> Varchar,
        #[max_length = 64]
        value -> Varchar,
    }
}

diesel::table! {
    stack_labels (id) {
        id -> Uuid,
        stack_id -> Uuid,
        #[max_length = 64]
        label -> Varchar,
    }
}

diesel::table! {
    stacks (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        generator_id -> Uuid,
    }
}

diesel::table! {
    stack_templates (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        generator_id -> Nullable<Uuid>,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        version -> Int4,
        template_content -> Text,
        parameters_schema -> Text,
        #[max_length = 64]
        checksum -> Varchar,
    }
}

diesel::table! {
    template_labels (id) {
        id -> Uuid,
        template_id -> Uuid,
        #[max_length = 64]
        label -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    template_annotations (id) {
        id -> Uuid,
        template_id -> Uuid,
        #[max_length = 64]
        key -> Varchar,
        #[max_length = 64]
        value -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    template_targets (id) {
        id -> Uuid,
        template_id -> Uuid,
        stack_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    rendered_deployment_objects (id) {
        id -> Uuid,
        deployment_object_id -> Uuid,
        template_id -> Uuid,
        template_version -> Int4,
        template_parameters -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    work_orders (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 50]
        work_type -> Varchar,
        yaml_content -> Text,
        #[max_length = 20]
        status -> Varchar,
        claimed_by -> Nullable<Uuid>,
        claimed_at -> Nullable<Timestamptz>,
        claim_timeout_seconds -> Int4,
        max_retries -> Int4,
        retry_count -> Int4,
        backoff_seconds -> Int4,
        next_retry_after -> Nullable<Timestamptz>,
        last_error -> Nullable<Text>,
        last_error_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    work_order_log (id) {
        id -> Uuid,
        #[max_length = 50]
        work_type -> Varchar,
        created_at -> Timestamptz,
        claimed_at -> Nullable<Timestamptz>,
        completed_at -> Timestamptz,
        claimed_by -> Nullable<Uuid>,
        success -> Bool,
        retries_attempted -> Int4,
        result_message -> Nullable<Text>,
        yaml_content -> Text,
    }
}

diesel::table! {
    work_order_targets (id) {
        id -> Uuid,
        work_order_id -> Uuid,
        agent_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    work_order_labels (id) {
        id -> Uuid,
        work_order_id -> Uuid,
        #[max_length = 64]
        label -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    work_order_annotations (id) {
        id -> Uuid,
        work_order_id -> Uuid,
        #[max_length = 64]
        key -> Varchar,
        #[max_length = 64]
        value -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    diagnostic_requests (id) {
        id -> Uuid,
        agent_id -> Uuid,
        deployment_object_id -> Uuid,
        #[max_length = 20]
        status -> Varchar,
        #[max_length = 255]
        requested_by -> Nullable<Varchar>,
        created_at -> Timestamptz,
        claimed_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    diagnostic_results (id) {
        id -> Uuid,
        request_id -> Uuid,
        pod_statuses -> Text,
        events -> Text,
        log_tails -> Nullable<Text>,
        collected_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    webhook_subscriptions (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        url_encrypted -> Bytea,
        auth_header_encrypted -> Nullable<Bytea>,
        event_types -> Array<Nullable<Text>>,
        filters -> Nullable<Text>,
        enabled -> Bool,
        max_retries -> Int4,
        timeout_seconds -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        created_by -> Nullable<Varchar>,
    }
}

diesel::table! {
    webhook_deliveries (id) {
        id -> Uuid,
        subscription_id -> Uuid,
        #[max_length = 100]
        event_type -> Varchar,
        event_id -> Uuid,
        payload -> Text,
        #[max_length = 20]
        status -> Varchar,
        attempts -> Int4,
        last_attempt_at -> Nullable<Timestamptz>,
        next_attempt_at -> Nullable<Timestamptz>,
        last_error -> Nullable<Text>,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    audit_logs (id) {
        id -> Uuid,
        timestamp -> Timestamptz,
        #[max_length = 20]
        actor_type -> Varchar,
        actor_id -> Nullable<Uuid>,
        #[max_length = 100]
        action -> Varchar,
        #[max_length = 50]
        resource_type -> Varchar,
        resource_id -> Nullable<Uuid>,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(agent_annotations -> agents (agent_id));
diesel::joinable!(agent_events -> agents (agent_id));
diesel::joinable!(agent_events -> deployment_objects (deployment_object_id));
diesel::joinable!(deployment_health -> agents (agent_id));
diesel::joinable!(deployment_health -> deployment_objects (deployment_object_id));
diesel::joinable!(agent_labels -> agents (agent_id));
diesel::joinable!(agent_targets -> agents (agent_id));
diesel::joinable!(agent_targets -> stacks (stack_id));
diesel::joinable!(deployment_objects -> stacks (stack_id));
diesel::joinable!(stack_annotations -> stacks (stack_id));
diesel::joinable!(stack_labels -> stacks (stack_id));
diesel::joinable!(stacks -> generators (generator_id));
diesel::joinable!(stack_templates -> generators (generator_id));
diesel::joinable!(template_labels -> stack_templates (template_id));
diesel::joinable!(template_annotations -> stack_templates (template_id));
diesel::joinable!(template_targets -> stack_templates (template_id));
diesel::joinable!(template_targets -> stacks (stack_id));
diesel::joinable!(rendered_deployment_objects -> deployment_objects (deployment_object_id));
diesel::joinable!(rendered_deployment_objects -> stack_templates (template_id));
diesel::joinable!(work_orders -> agents (claimed_by));
diesel::joinable!(work_order_log -> agents (claimed_by));
diesel::joinable!(work_order_targets -> work_orders (work_order_id));
diesel::joinable!(work_order_targets -> agents (agent_id));
diesel::joinable!(work_order_labels -> work_orders (work_order_id));
diesel::joinable!(work_order_annotations -> work_orders (work_order_id));
diesel::joinable!(diagnostic_requests -> agents (agent_id));
diesel::joinable!(diagnostic_requests -> deployment_objects (deployment_object_id));
diesel::joinable!(diagnostic_results -> diagnostic_requests (request_id));
diesel::joinable!(webhook_deliveries -> webhook_subscriptions (subscription_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_role,
    agent_annotations,
    agent_events,
    agent_labels,
    agent_targets,
    agents,
    app_initialization,
    audit_logs,
    deployment_health,
    deployment_objects,
    diagnostic_requests,
    diagnostic_results,
    generators,
    rendered_deployment_objects,
    stack_annotations,
    stack_labels,
    stack_templates,
    stacks,
    template_annotations,
    template_labels,
    template_targets,
    work_orders,
    work_order_annotations,
    work_order_labels,
    work_order_log,
    work_order_targets,
    webhook_subscriptions,
    webhook_deliveries,
);
