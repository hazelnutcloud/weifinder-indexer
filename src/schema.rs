// @generated automatically by Diesel CLI.

diesel::table! {
    checkpoints (chain_id) {
        chain_id -> Integer,
        last_saved_block_number -> Nullable<Integer>,
    }
}
