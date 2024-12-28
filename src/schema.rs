// @generated automatically by Diesel CLI.

diesel::table! {
    predictor_saves (id) {
        id -> Integer,
        save -> Binary,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
