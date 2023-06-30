use std::sync::Arc;

use crate::{event::Event, persistence::InMemoryTransaction};

use super::{LogEvent, Offset, Reader, SharedReader, SharedWriter, WithOffset, Writer};

use async_condvar_fair::Condvar;
use futures::FutureExt;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

type InMemoryLogInner = Vec<Event>;

pub struct InMemoryLog {
    inner: Arc<RwLock<InMemoryLogInner>>,
    condvar: Arc<Condvar>,
    runtime: tokio::runtime::Runtime,
}

impl InMemoryLog {
    pub async fn read(&self) -> OwnedRwLockReadGuard<InMemoryLogInner> {
        self.inner.clone().read_owned().await
    }

    pub async fn write(&self) -> OwnedRwLockWriteGuard<InMemoryLogInner> {
        self.inner.clone().write_owned().await
    }

    async fn write_events(&self, events: &[Event]) -> eyre::Result<Offset> {
        let mut write = self.write().await;

        write.extend_from_slice(events);
        self.condvar.notify_all();

        Ok(u64::try_from(write.len())?)
    }
}
impl Reader for InMemoryLog {
    fn read(
        &self,
        conn: &mut dyn crate::persistence::Connection,
        offset: Offset,
        limit: usize,
        timeout: Option<std::time::Duration>,
    ) -> eyre::Result<super::WithOffset<Vec<super::LogEvent>>> {
        let offset_usize = usize::try_from(offset)?;
        let condvar = self.condvar.clone();

        let read = self.runtime.block_on(async {
            let read = self.read().await;

            if read.len() == offset_usize {
                if let Some(timeout) = timeout {
                    let timeout_fut = async move {
                        tokio::time::sleep(timeout).await;
                        condvar.notify_all();
                    }
                    .fuse();
                    let wait_fut = self.condvar.wait((read, self.inner.clone())).fuse();
                    let mut timeout_fut = Box::pin(timeout_fut);
                    let mut wait_fut = Box::pin(wait_fut);

                    loop {
                        futures::select! {
                            read = wait_fut => break read,
                            _ = timeout_fut => {},
                        };
                    }
                } else {
                    self.condvar.wait((read, self.inner.clone())).await
                }
            } else {
                read
            }
        });

        let res: Vec<_> = read
            .get(offset_usize..)
            .ok_or_else(|| eyre::format_err!("out of bounds"))?
            .iter()
            .take(limit)
            .enumerate()
            .map(|(i, e)| LogEvent {
                offset: offset + u64::try_from(i).expect("no fail"),
                details: e.clone(),
            })
            .collect();

        Ok(WithOffset {
            offset: offset + u64::try_from(res.len()).expect("no fail"),
            data: res,
        })
    }

    fn get_start_offset(&self) -> eyre::Result<Offset> {
        Ok(0)
    }
}

impl Writer for InMemoryLog {
    fn write_tr(
        &self,
        conn: &mut dyn crate::persistence::Transaction<'_>,
        events: &[Event],
    ) -> eyre::Result<Offset> {
        conn.cast().as_mut::<InMemoryTransaction>()?;
        futures::executor::block_on(self.write_events(events))
    }
}
pub fn new_in_memory_shared() -> eyre::Result<(SharedWriter, SharedReader)> {
    let log = Arc::new(InMemoryLog {
        inner: Arc::new(RwLock::new(Vec::new())),
        condvar: Arc::new(Condvar::default()),
        runtime: tokio::runtime::Runtime::new()?,
    });

    Ok((log.clone(), log))
}
