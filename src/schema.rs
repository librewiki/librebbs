table! {
    boards (id) {
        id -> Integer,
        display_name -> Varchar,
        name -> Varchar,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    comments (id) {
        id -> Integer,
        topic_id -> Integer,
        content -> Mediumtext,
        author_id -> Nullable<Integer>,
        author_name -> Nullable<Varchar>,
        author_ip -> Varbinary,
        is_hidden -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    logs (id) {
        id -> Integer,
        log_type_id -> Integer,
        content -> Varchar,
        user_id -> Nullable<Integer>,
        user_name -> Nullable<Varchar>,
        user_ip -> Varbinary,
        created_at -> Timestamp,
    }
}

table! {
    log_types (id) {
        id -> Integer,
        name -> Nullable<Varchar>,
    }
}

table! {
    topics (id) {
        id -> Integer,
        board_id -> Integer,
        title -> Varchar,
        author_id -> Nullable<Integer>,
        author_name -> Nullable<Varchar>,
        author_ip -> Varbinary,
        is_closed -> Bool,
        is_suspended -> Bool,
        is_hidden -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(comments -> topics (topic_id));
joinable!(logs -> log_types (log_type_id));
joinable!(topics -> boards (board_id));

allow_tables_to_appear_in_same_query!(
    boards,
    comments,
    logs,
    log_types,
    topics,
);
