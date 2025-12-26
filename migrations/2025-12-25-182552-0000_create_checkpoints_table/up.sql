-- Your SQL goes here
CREATE TABLE checkpoints (
  chain_id INTEGER NOT NULL PRIMARY KEY,
  last_saved_block_number INTEGER
)
