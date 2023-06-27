mod in_memory;

pub use self::in_memory::{InMemoryConnection, InMemoryPersistence, InMemoryTransaction};

use eyre::Result;
use std::{any::Any, sync::Arc};
/// An interface of any persistence
///
/// Persistence is anything that a Repository implementation could
/// use to store data.
pub trait Persistence: Send + Sync {
    /// Get a connection to persistence
    fn get_connection(&self) -> Result<OwnedConnection>;
}

pub type SharedPersistence = Arc<dyn Persistence>;

pub trait Connection: Any {
    fn start_transaction(&mut self) -> Result<OwnedTransaction<'_>>;

    // fn cast(&mut self) -> Caster<'_, 'static>;
}

pub type OwnedConnection = Box<dyn Connection>;

pub trait Transaction<'a> {
    fn commit(self: Box<Self>) -> Result<()>;
    fn rollback(self: Box<Self>) -> Result<()>;

    // fn cast<'b>(&'b mut self) -> Caster<'b, 'a>
    // where
    //     'a: 'b;
}

pub type OwnedTransaction<'a> = Box<dyn Transaction<'a> + 'a>;
