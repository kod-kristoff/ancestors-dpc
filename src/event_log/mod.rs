mod in_memory;
use std::sync::Arc;

use crate::{
    event::Event,
    persistence::{Connection, Transaction},
};

pub use self::in_memory::new_in_memory_shared;
pub type Offset = u64;

pub trait Reader {
    fn get_start_offset(&self) -> eyre::Result<Offset>;
}

pub trait Writer {
    fn write(&self, conn: &mut dyn Connection, events: &[Event]) -> eyre::Result<Offset>;
}

pub type SharedReader = Arc<dyn Reader + Send + Sync + 'static>;
pub type SharedWriter = Arc<dyn Writer + Send + Sync + 'static>;
