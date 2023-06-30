mod in_memory;

pub use self::in_memory::{InMemoryConnection, InMemoryPersistence, InMemoryTransaction};

use dyno::{Tag, Tagged};

use eyre::Result;
use std::{any::Any, fmt, sync::Arc};
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

    fn cast(&mut self) -> Caster<'_, 'static>;
}

pub type OwnedConnection = Box<dyn Connection>;

pub trait Transaction<'a> {
    fn commit(self: Box<Self>) -> Result<()>;
    fn rollback(self: Box<Self>) -> Result<()>;

    fn cast<'b>(&'b mut self) -> Caster<'b, 'a>
    where
        'a: 'b;
}

pub type OwnedTransaction<'a> = Box<dyn Transaction<'a> + 'a>;

#[derive(Debug)]
pub enum Error {
    // #[error("wrong tyxdpe")]
    WrongType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongType => write!(f, "wrong type"),
        }
    }
}

impl std::error::Error for Error {}

/// Dynamic cast helper
///
/// This struct allows an implementation of a Repository
/// to cast at runtime a type-erased [`Transaction`] or [`Connection`]
/// instance to back to a concrete type that it needs and expects.
///
/// # Safety
/// See https://users.rust-lang.org/t/help-with-using-any-to-cast-t-a-back-and-forth/69900/8
///
/// The safety is enforced by the fact that `Caster` pinky-promises to never
/// allow any reference other that `&'caster mut T` out of itself, and
/// `'a` must always outlive `'caster` or borrowck will be upset.
pub struct Caster<'borrow, 'value>(&'borrow mut (dyn Tagged<'value> + 'value));

impl<'borrow, 'value> Caster<'borrow, 'value> {
    pub fn new<I: Tag<'value>>(any: &'borrow mut I::Type) -> Self {
        Self(<dyn Tagged>::tag_mut::<I>(any))
    }

    // Returns `Result` so it's easier to handle with ? than an option
    pub fn as_mut<I: Tag<'value>>(&'borrow mut self) -> Result<&'borrow mut I::Type, Error> {
        self.0.downcast_mut::<I>().ok_or(Error::WrongType)
    }
}
