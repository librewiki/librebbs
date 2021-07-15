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
    posts (id) {
        id -> Integer,
        board_id -> Integer,
        title -> Varchar,
        content -> Mediumtext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(posts -> boards (board_id));

allow_tables_to_appear_in_same_query!(
    boards,
    posts,
);
