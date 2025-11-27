use std::fmt::Display;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Mutex::new(Shared::<T> {
        value: None,
        waker: None,
        closed: false,
    }));

    let tx = Sender::<T> {
        inner: shared.clone(),
    };

    let rx = Receiver::<T> {
        inner: shared.clone(),
    };

    return (tx, rx);
}

pub struct Receiver<T> {
    inner: Arc<Mutex<Shared<T>>>,
}

pub struct Sender<T> {
    inner: Arc<Mutex<Shared<T>>>,
}

struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
    closed: bool,
}

impl<T> Sender<T> {
    fn send(&self, value: T) -> Result<()> {
        let mut shared = self.inner.lock().unwrap();
        if shared.closed {
            return Err(Error::ChannelClosed);
        }

        if shared.value.is_some() {
            return Err(Error::DuplicateSend);
        }

        shared.value = Some(value);
        if let Some(waker) = shared.waker.take() {
            waker.wake();
        }

        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut shared = self.inner.lock().unwrap();
        shared.closed = true;
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.inner.lock().unwrap();
        if shared.closed {
            return Poll::Ready(Err(Error::ChannelClosed));
        }

        if let Some(value) = shared.value.take() {
            shared.closed = true;
            return Poll::Ready(Ok(value));
        }

        shared.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut shared = self.inner.lock().unwrap();
        shared.closed = true;
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ChannelClosed,
    DuplicateSend,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ChannelClosed => write!(f, "Channel is closed"),
            Error::DuplicateSend => write!(f, "Duplicate send on oneshot channel"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_send() {
        let (tx, rx) = oneshot::<usize>();
        tx.send(42).unwrap();
        assert_eq!(42, rx.await.unwrap());
    }

    #[tokio::test]
    async fn test_duplicate_send() {
        let (tx, rx) = oneshot::<usize>();
        tx.send(42).unwrap();
        assert_eq!(tx.send(42), Err(Error::DuplicateSend));
        assert_eq!(42, rx.await.unwrap());
    }

    #[tokio::test]
    async fn test_channel_closed_after_recv() {
        let (tx, rx) = oneshot::<usize>();
        tx.send(42).unwrap();
        assert_eq!(42, rx.await.unwrap());
        assert_eq!(Err(Error::ChannelClosed), tx.send(43));
    }

    #[tokio::test]
    async fn test_channel_closed_after_drop_tx() {
        let (tx, rx) = oneshot::<usize>();
        drop(tx);
        assert_eq!(Err(Error::ChannelClosed), rx.await);
    }

    #[tokio::test]
    async fn test_channel_closed_after_drop_rx() {
        let (tx, rx) = oneshot::<usize>();
        drop(rx);
        assert_eq!(Err(Error::ChannelClosed), tx.send(42));
    }

    #[tokio::test]
    async fn test_double_drop() {
        let (tx, rx) = oneshot::<usize>();
        drop(rx);
        drop(tx);
    }
}
