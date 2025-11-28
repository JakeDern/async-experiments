use std::cell::RefCell;
use std::fmt::Display;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

pub fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Rc::new(RefCell::new(Shared::<T> {
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
    inner: Rc<RefCell<Shared<T>>>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        let mut shared = self.inner.borrow_mut();
        let value = shared.value.take();
        match value {
            None => None,
            Some(x) => {
                shared.closed = true;
                Some(x)
            }
        }
    }
}

pub struct Sender<T> {
    inner: Rc<RefCell<Shared<T>>>,
}

struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
    closed: bool,
}

impl<T> Sender<T> {
    pub fn send(self, value: T) -> Result<()> {
        let mut shared = self.inner.borrow_mut();
        shared.value = Some(value);
        if let Some(waker) = shared.waker.take() {
            waker.wake();
        }

        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut shared = self.inner.borrow_mut();
        shared.closed = true;
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.inner.borrow_mut();
        if let Some(value) = shared.value.take() {
            shared.closed = true;
            return Poll::Ready(Ok(value));
        }

        if shared.closed {
            return Poll::Ready(Err(Error::ChannelClosed));
        }

        shared.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        println!("Dropping receiver");
        let mut shared = self.inner.borrow_mut();
        shared.closed = true;
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ChannelClosed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ChannelClosed => write!(f, "Channel is closed"),
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
        assert_eq!(42, rx.await.unwrap());
    }

    #[tokio::test]
    async fn test_channel_closed_after_recv() {
        let (tx, rx) = oneshot::<usize>();
        tx.send(42).unwrap();
        assert_eq!(42, rx.await.unwrap());
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

    #[tokio::test]
    async fn test_send_drop_recv() {
        let (tx, rx) = oneshot::<usize>();
        tx.send(42).unwrap();
        assert_eq!(42, rx.await.unwrap());
    }
}
