// @generated automatically by Diesel CLI.

diesel::table! {
    cardhistory (card_id) {
        card_id -> Integer,
        review_time -> Timestamp,
        user_response -> Nullable<Integer>,
        duration_to_respond -> Nullable<Integer>,
        box_position_initial -> Nullable<Integer>,
    }
}

diesel::table! {
    cards (id) {
        id -> Integer,
        front -> Text,
        back -> Text,
        queue_score -> Nullable<Integer>,
        box_position -> Integer,
    }
}

diesel::table! {
    deadlines (id) {
        id -> Integer,
        date_created -> Timestamp,
        deadline_date -> Timestamp,
        study_intensity -> Nullable<Integer>,
        num_reset -> Integer,
    }
}

diesel::table! {
    deckitems (item_id) {
        item_id -> Integer,
        deck_id -> Integer,
    }
}

diesel::table! {
    decks (id) {
        id -> Integer,
        date_created -> Timestamp,
        num_boxes -> Integer,
    }
}

diesel::table! {
    documents (id) {
        id -> Integer,
        source_text -> Nullable<Binary>,
        notes -> Nullable<Text>,
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
    media (id) {
        id -> Integer,
        content -> Nullable<Binary>,
        entry_id -> Nullable<Integer>,
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
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cardhistory,
    cards,
    deadlines,
    deckitems,
    decks,
    documents,
    entries,
    folders,
    media,
    parents,
    quotas,
    userconfig,
);
