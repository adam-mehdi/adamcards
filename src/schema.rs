// @generated automatically by Diesel CLI.

diesel::table! {
    ankiquotas (deck_id, date_practiced) {
        deck_id -> Integer,
        date_practiced -> Date,
        new_practiced -> Integer,
        review_practiced -> Integer,
    }
}

diesel::table! {
    cards (id) {
        id -> Integer,
        deck_id -> Integer,
        front -> Text,
        back -> Text,
        queue_score -> Nullable<Integer>,
        box_position -> Nullable<Integer>,
        repetitions -> Nullable<Integer>,
        easiness -> Nullable<Float>,
        interval -> Nullable<Integer>,
        next_practice -> Nullable<Date>,
        rephrasing1 -> Nullable<Text>,
        rephrasing2 -> Nullable<Text>,
        rephrasing3 -> Nullable<Text>,
        rephrasing4 -> Nullable<Text>,
        rephrasing5 -> Nullable<Text>,
        explanation -> Nullable<Text>,
    }
}

diesel::table! {
    deadlines (id) {
        id -> Integer,
        deadline_date -> Nullable<Timestamp>,
        study_intensity -> Nullable<Integer>,
        num_reset -> Nullable<Integer>,
        is_anki -> Bool,
    }
}

diesel::table! {
    decks (id) {
        id -> Integer,
        num_boxes -> Nullable<Integer>,
        new_per_day -> Nullable<Integer>,
    }
}

diesel::table! {
    entries (id) {
        id -> Integer,
        name -> Text,
        is_expanded -> Nullable<Bool>,
    }
}

diesel::table! {
    folders (id) {
        id -> Integer,
    }
}

diesel::table! {
    parents (parent_id, child_id) {
        parent_id -> Integer,
        child_id -> Integer,
    }
}

diesel::table! {
    quotas (id, days_to_go) {
        id -> Integer,
        days_to_go -> Integer,
        new_assigned -> Integer,
        review_assigned -> Integer,
        new_quota_initial -> Integer,
        review_quota_initial -> Integer,
        new_practiced -> Integer,
        review_practiced -> Integer,
    }
}

diesel::table! {
    userconfig (config_id) {
        config_id -> Integer,
        is_dark_mode -> Bool,
        is_text_field -> Bool,
        api_key -> Nullable<Text>,
    }
}

diesel::joinable!(ankiquotas -> decks (deck_id));
diesel::joinable!(cards -> decks (deck_id));
diesel::joinable!(deadlines -> entries (id));
diesel::joinable!(decks -> entries (id));
diesel::joinable!(folders -> entries (id));

diesel::allow_tables_to_appear_in_same_query!(
    ankiquotas,
    cards,
    deadlines,
    decks,
    entries,
    folders,
    parents,
    quotas,
    userconfig,
);
