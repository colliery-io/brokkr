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

diesel::joinable!(agent_annotations -> agents (agent_id));
diesel::joinable!(agent_events -> agents (agent_id));
diesel::joinable!(agent_events -> deployment_objects (deployment_object_id));
diesel::joinable!(agent_labels -> agents (agent_id));
diesel::joinable!(agent_targets -> agents (agent_id));
diesel::joinable!(agent_targets -> stacks (stack_id));
diesel::joinable!(deployment_objects -> stacks (stack_id));
diesel::joinable!(stack_annotations -> stacks (stack_id));
diesel::joinable!(stack_labels -> stacks (stack_id));
diesel::joinable!(stacks -> generators (generator_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_role,
    agent_annotations,
    agent_events,
    agent_labels,
    agent_targets,
    agents,
    app_initialization,
    deployment_objects,
    generators,
    stack_annotations,
    stack_labels,
    stacks,
);
