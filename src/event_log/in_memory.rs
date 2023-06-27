use std::sync::Arc;

use crate::event::Event;

use super::{SharedReader, SharedWriter};

use async_condvar_fair::Condvar;
use futures::FutureExt;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

type InMemoryLogInner = Vec<Event>;

pub struct InMemoryLog {
    inner: Arc<RwLock<InMemoryLogInner>>,
    condvar: Arc<Condvar>,
    runtime: tokio::runtime::Runtime,
}

pub fn new_in_memory_shared() -> eyre::Result<(SharedWriter, SharedReader)> {
    let log = Arc::new(InMemoryLog {
        inner: Arc::new(RwLock::new(Vec::new())),
        condvar: Arc::new(Condvar::default()),
        runtime: tokio::runtime::Runtime::new(),
    });

    Ok((log.clone(), log))
}
