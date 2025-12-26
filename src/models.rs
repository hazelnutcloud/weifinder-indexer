use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::checkpoints)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Checkpoint {
    pub last_saved_block_number: Option<i32>,
}
