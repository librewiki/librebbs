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
        author_ip -> Binary,
        is_hidden -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    topics (id) {
        id -> Integer,
        board_id -> Integer,
        title -> Varchar,
        author_id -> Nullable<Integer>,
        author_name -> Nullable<Varchar>,
        author_ip -> Binary,
        is_closed -> Bool,
        is_suspended -> Bool,
        is_hidden -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(comments -> topics (topic_id));
joinable!(topics -> boards (board_id));

allow_tables_to_appear_in_same_query!(
    boards,
    comments,
    topics,
);
