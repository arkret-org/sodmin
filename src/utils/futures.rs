//! Minimal `join_all` shim.
//!
//! The `futures` crate isn't in the dep set and adding it for one
//! combinator is overkill. Polls every future in the batch concurrently
//! by maintaining a Vec of pinned futures and looping through them
//! until each yields a value. `wasm32` is single-threaded so there's
//! no need for `Send` bounds; the futures share the JS event-loop
//! reactor and `gloo-net` requests run in parallel because the browser
//! `fetch()` is dispatched on first poll.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

enum Slot<F, T>
where
    F: Future<Output = T>,
{
    Pending(Pin<Box<F>>),
    Done(Option<T>),
}

struct JoinAll<F, T>
where
    F: Future<Output = T>,
{
    slots: Vec<Slot<F, T>>,
}

impl<F, T> Future for JoinAll<F, T>
where
    F: Future<Output = T>,
{
    type Output = Vec<T>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: we never move out of `slots`; only mutate it in place.
        let this = unsafe { self.get_unchecked_mut() };
        let mut all_done = true;
        for slot in this.slots.iter_mut() {
            if let Slot::Pending(fut) = slot {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(v) => {
                        *slot = Slot::Done(Some(v));
                    }
                    Poll::Pending => {
                        all_done = false;
                    }
                }
            }
        }
        if all_done {
            let out: Vec<T> = this
                .slots
                .iter_mut()
                .map(|s| match s {
                    Slot::Done(v) => v.take().expect("value already taken"),
                    Slot::Pending(_) => unreachable!("all_done implies no Pending"),
                })
                .collect();
            Poll::Ready(out)
        } else {
            Poll::Pending
        }
    }
}

/// Polls all futures concurrently, returning their results in the
/// original order once every one completes.
pub async fn join_all<F, T>(iter: Vec<F>) -> Vec<T>
where
    F: Future<Output = T>,
{
    let join = JoinAll {
        slots: iter
            .into_iter()
            .map(|f| Slot::Pending(Box::pin(f)))
            .collect(),
    };
    join.await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_join_all() {
        let fut = join_all::<std::future::Ready<()>, ()>(Vec::new());
        // Drive the future to completion synchronously.
        let waker = futures_test_waker();
        let mut cx = Context::from_waker(&waker);
        let mut pinned = Box::pin(fut);
        let Poll::Ready(out) = pinned.as_mut().poll(&mut cx) else {
            panic!("expected ready");
        };
        assert!(out.is_empty());
    }

    #[test]
    fn join_preserves_order() {
        let f1 = std::future::ready(1);
        let f2 = std::future::ready(2);
        let f3 = std::future::ready(3);
        let fut = join_all(vec![f1, f2, f3]);
        let waker = futures_test_waker();
        let mut cx = Context::from_waker(&waker);
        let mut pinned = Box::pin(fut);
        let Poll::Ready(out) = pinned.as_mut().poll(&mut cx) else {
            panic!("expected ready");
        };
        assert_eq!(out, vec![1, 2, 3]);
    }

    fn futures_test_waker() -> std::task::Waker {
        use std::task::{RawWaker, RawWakerVTable, Waker};
        fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        fn noop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }
}
