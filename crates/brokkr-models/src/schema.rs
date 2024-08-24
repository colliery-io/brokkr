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
    agents (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        cluster_name -> Varchar,
        labels -> Nullable<Jsonb>,
        annotations -> Nullable<Jsonb>,
        last_heartbeat -> Nullable<Timestamptz>,
        #[max_length = 50]
        status -> Varchar,
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
    stacks (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        labels -> Nullable<Jsonb>,
        annotations -> Nullable<Jsonb>,
        agent_target -> Nullable<Jsonb>,
    }
}

diesel::joinable!(agent_events -> agents (agent_id));
diesel::joinable!(agent_events -> deployment_objects (deployment_object_id));
diesel::joinable!(deployment_objects -> stacks (stack_id));

diesel::allow_tables_to_appear_in_same_query!(
    agent_events,
    agents,
    deployment_objects,
    stacks,
);
