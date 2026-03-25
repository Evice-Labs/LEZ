use crate::error::DbError;

pub mod error;
pub mod indexer;
pub mod sequencer;
pub mod storable_cell;

pub type DbResult<T> = Result<T, DbError>;

// General

/// Maximal size of stored blocks in base.
///
/// Used to control db size.
///
/// Currently effectively unbounded.
pub const BUFF_SIZE_ROCKSDB: usize = usize::MAX;

/// Size of stored blocks cache in memory.
///
/// Keeping small to not run out of memory.
pub const CACHE_SIZE: usize = 1000;

/// Key base for storing metainformation which describe if first block has been set.
pub const DB_META_FIRST_BLOCK_SET_KEY: &str = "first_block_set";
/// Key base for storing metainformation about id of first block in db.
pub const DB_META_FIRST_BLOCK_IN_DB_KEY: &str = "first_block_in_db";
/// Key base for storing metainformation about id of last current block in db.
pub const DB_META_LAST_BLOCK_IN_DB_KEY: &str = "last_block_in_db";

/// Interval between state breakpoints.
pub const BREAKPOINT_INTERVAL: u8 = 100;

/// Name of block column family.
pub const CF_BLOCK_NAME: &str = "cf_block";
/// Name of meta column family.
pub const CF_META_NAME: &str = "cf_meta";

// Indexer-specific

/// Key base for storing metainformation about id of last observed L1 lib header in db.
pub const DB_META_LAST_OBSERVED_L1_LIB_HEADER_ID_IN_DB_KEY: &str =
    "last_observed_l1_lib_header_in_db";
/// Key base for storing metainformation about the last breakpoint.
pub const DB_META_LAST_BREAKPOINT_ID: &str = "last_breakpoint_id";

/// Name of breakpoint column family.
pub const CF_BREAKPOINT_NAME: &str = "cf_breakpoint";
/// Name of hash to id map column family.
pub const CF_HASH_TO_ID: &str = "cf_hash_to_id";
/// Name of tx hash to id map column family.
pub const CF_TX_TO_ID: &str = "cf_tx_to_id";
/// Name of account meta column family.
pub const CF_ACC_META: &str = "cf_acc_meta";
/// Name of account id to tx hash map column family.
pub const CF_ACC_TO_TX: &str = "cf_acc_to_tx";

// Sequencer-specific

/// Key base for storing metainformation about the last finalized block on Bedrock.
pub const DB_META_LAST_FINALIZED_BLOCK_ID: &str = "last_finalized_block_id";
/// Key base for storing metainformation about the latest block meta.
pub const DB_META_LATEST_BLOCK_META_KEY: &str = "latest_block_meta";

/// Key base for storing the NSSA state.
pub const DB_NSSA_STATE_KEY: &str = "nssa_state";

/// Name of state column family.
pub const CF_NSSA_STATE_NAME: &str = "cf_nssa_state";
