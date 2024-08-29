// @generated automatically by Diesel CLI.

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
    agent_targets (id) {
        id -> Uuid,
        stack_id -> Uuid,
        agent_id -> Uuid,
        created_at -> Timestamptz,
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
    annotations (id) {
        id -> Uuid,
        object_id -> Uuid,
        #[max_length = 50]
        object_type -> Varchar,
        #[max_length = 255]
        key -> Varchar,
        value -> Text,
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
    labels (id) {
        id -> Uuid,
        object_id -> Uuid,
        #[max_length = 50]
        object_type -> Varchar,
        #[max_length = 255]
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
    }
}

diesel::joinable!(agent_events -> agents (agent_id));
diesel::joinable!(agent_events -> deployment_objects (deployment_object_id));
diesel::joinable!(agent_targets -> agents (agent_id));
diesel::joinable!(agent_targets -> stacks (stack_id));
diesel::joinable!(deployment_objects -> stacks (stack_id));

diesel::allow_tables_to_appear_in_same_query!(
    agent_events,
    agent_targets,
    agents,
    annotations,
    deployment_objects,
    labels,
    stacks,
);
