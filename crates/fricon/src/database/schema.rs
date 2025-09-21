// @generated automatically by Diesel CLI.

diesel::table! {
    datasets (id) {
        id -> Integer,
        uuid -> Text,
        name -> Text,
        description -> Text,
        favorite -> Bool,
        status -> Text,
        index_columns -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    datasets_tags (dataset_id, tag_id) {
        dataset_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(datasets_tags -> datasets (dataset_id));
diesel::joinable!(datasets_tags -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(datasets, datasets_tags, tags,);
