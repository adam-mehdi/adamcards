// @generated automatically by Diesel CLI.

diesel::table! {
    cardhistory (card_id) {
        card_id -> Int4,
        review_time -> Timestamptz,
        user_response -> Nullable<Int4>,
        duration_to_respond -> Nullable<Int4>,
        box_position_initial -> Nullable<Int4>,
    }
}

diesel::table! {
    cards (id) {
        id -> Int4,
        front -> Text,
        back -> Text,
        queue_score -> Nullable<Int4>,
        box_position -> Int4,
    }
}

diesel::table! {
    deadlines (id) {
        id -> Int4,
        date_created -> Timestamptz,
        deadline_date -> Timestamptz,
        study_intensity -> Nullable<Int4>,
        num_reset -> Int4,
    }
}

diesel::table! {
    deckitem (item_id) {
        item_id -> Int4,
        deck_id -> Nullable<Int4>,
    }
}

diesel::table! {
    deckitems (item_id) {
        item_id -> Int4,
        deck_id -> Int4,
    }
}

diesel::table! {
    decks (id) {
        id -> Int4,
        date_created -> Timestamptz,
        num_boxes -> Int4,
    }
}

diesel::table! {
    documents (id) {
        id -> Int4,
        source_text -> Nullable<Bytea>,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    entries (id) {
        id -> Int4,
        name -> Varchar,
        is_expanded -> Nullable<Bool>,
    }
}

diesel::table! {
    folders (id) {
        id -> Int4,
    }
}

diesel::table! {
    media (id) {
        id -> Int4,
        content -> Nullable<Bytea>,
        entry_id -> Nullable<Int4>,
    }
}

diesel::table! {
    parents (parent_id, child_id) {
        parent_id -> Int4,
        child_id -> Int4,
    }
}

diesel::table! {
    quotas (id, days_to_go) {
        id -> Int4,
        days_to_go -> Int4,
        new_assigned -> Int4,
        review_assigned -> Int4,
        new_quota_initial -> Int4,
        review_quota_initial -> Int4,
        new_practiced -> Int4,
        review_practiced -> Int4,
    }
}

diesel::table! {
    userconfig (config_id) {
        config_id -> Int4,
        is_dark_mode -> Bool,
        is_text_field -> Bool,
    }
}

diesel::joinable!(cardhistory -> cards (card_id));
diesel::joinable!(cards -> deckitems (id));
diesel::joinable!(deadlines -> entries (id));
diesel::joinable!(deckitems -> decks (deck_id));
diesel::joinable!(decks -> entries (id));
diesel::joinable!(documents -> deckitems (id));
diesel::joinable!(folders -> entries (id));
diesel::joinable!(media -> deckitems (entry_id));
diesel::joinable!(quotas -> decks (id));

diesel::allow_tables_to_appear_in_same_query!(
    cardhistory,
    cards,
    deadlines,
    deckitem,
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
