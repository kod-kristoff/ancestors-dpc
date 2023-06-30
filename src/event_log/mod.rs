mod in_memory;
use std::{sync::Arc, time::Duration};

use crate::{
    event::Event,
    persistence::{Connection, Transaction},
};

pub use self::in_memory::new_in_memory_shared;
pub type Offset = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEvent {
    pub offset: Offset,
    pub details: Event,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithOffset<T> {
    pub offset: Offset,
    pub data: T,
}

pub trait Reader {
    fn get_start_offset(&self) -> eyre::Result<Offset>;

    fn read(
        &self,
        conn: &mut dyn Connection,
        offset: Offset,
        limit: usize,
        timeout: Option<Duration>,
    ) -> eyre::Result<WithOffset<Vec<LogEvent>>>;

    fn read_one(
        &self,
        conn: &mut dyn Connection,
        offset: Offset,
    ) -> eyre::Result<WithOffset<Option<LogEvent>>> {
        let WithOffset { offset, data } =
            self.read(conn, offset, 1, Some(Duration::from_millis(0)))?;
        assert!(data.len() <= 1);
        Ok(WithOffset {
            offset,
            data: data.into_iter().next(),
        })
    }
}

pub trait Writer {
    fn write(&self, conn: &mut dyn Connection, events: &[Event]) -> eyre::Result<Offset> {
        self.write_tr(&mut *conn.start_transaction()?, events)
    }

    fn write_tr(&self, conn: &mut dyn Transaction<'_>, events: &[Event]) -> eyre::Result<Offset>;
}

pub type SharedReader = Arc<dyn Reader + Send + Sync + 'static>;
pub type SharedWriter = Arc<dyn Writer + Send + Sync + 'static>;
