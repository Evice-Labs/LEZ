use std::sync::Arc;

use borsh::{BorshDeserialize, BorshSerialize};
use rocksdb::{BoundColumnFamily, DBWithThreadMode, MultiThreaded, WriteBatch};

use crate::{DbResult, error::DbError};

pub mod cells;

pub trait SimpleStorableCell: BorshSerialize + BorshDeserialize {
    const CF_NAME: &'static str;
    const CELL_NAME: &'static str;

    fn key_constructor() -> DbResult<Vec<u8>>;
    fn value_constructor(&self) -> DbResult<Vec<u8>>;

    fn column_ref(db: &DBWithThreadMode<MultiThreaded>) -> Arc<BoundColumnFamily<'_>> {
        db.cf_handle(Self::CF_NAME)
            .unwrap_or_else(|| panic!("Column family {:?} must be present", Self::CF_NAME))
    }

    fn get(db: &DBWithThreadMode<MultiThreaded>) -> DbResult<Self> {
        let cf_ref = Self::column_ref(db);
        let res = db
            .get_cf(&cf_ref, Self::key_constructor()?)
            .map_err(|rerr| {
                DbError::rocksdb_cast_message(
                    rerr,
                    Some(format!("Failed to read {:?}", Self::CELL_NAME)),
                )
            })?;

        if let Some(data) = res {
            Ok(borsh::from_slice::<Self>(&data).map_err(|err| {
                DbError::borsh_cast_message(
                    err,
                    Some(format!("Failed to deserialize {:?}", Self::CELL_NAME)),
                )
            })?)
        } else {
            Err(DbError::db_interaction_error(format!(
                "{:?} not found",
                Self::CELL_NAME
            )))
        }
    }

    fn get_opt(db: &DBWithThreadMode<MultiThreaded>) -> DbResult<Option<Self>> {
        let cf_ref = Self::column_ref(db);
        let res = db
            .get_cf(&cf_ref, Self::key_constructor()?)
            .map_err(|rerr| {
                DbError::rocksdb_cast_message(
                    rerr,
                    Some(format!("Failed to read {:?}", Self::CELL_NAME)),
                )
            })?;

        res.map(|data| {
            borsh::from_slice::<Self>(&data).map_err(|err| {
                DbError::borsh_cast_message(
                    err,
                    Some(format!("Failed to deserialize {:?}", Self::CELL_NAME)),
                )
            })
        })
        .transpose()
    }

    fn put(&self, db: &DBWithThreadMode<MultiThreaded>) -> DbResult<()> {
        let cf_ref = Self::column_ref(db);
        db.put_cf(&cf_ref, Self::key_constructor()?, self.value_constructor()?)
            .map_err(|rerr| {
                DbError::rocksdb_cast_message(
                    rerr,
                    Some(format!("Failed to write {:?}", Self::CELL_NAME)),
                )
            })?;
        Ok(())
    }

    fn put_batch(
        &self,
        db: &DBWithThreadMode<MultiThreaded>,
        write_batch: &mut WriteBatch,
    ) -> DbResult<()> {
        let cf_ref = Self::column_ref(db);
        write_batch.put_cf(&cf_ref, Self::key_constructor()?, self.value_constructor()?);
        Ok(())
    }
}
