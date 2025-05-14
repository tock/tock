use core::{
    cell::Cell,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
};

use embedded_hal_async::delay::DelayNs;
use kernel::{
    debug,
    hil::time::{Alarm, AlarmClient, ConvertTicks},
};

enum State {
    Off,
    Sleeping,
}

pub struct Delay<'a, A: Alarm<'a>> {
    has_instance: AtomicBool,
    alarm: &'a A,
    waker: Cell<Option<Waker>>,
}

impl<'a, A: Alarm<'a>> Delay<'a, A> {
    pub fn new(alarm: &'a A) -> Delay<'a, A> {
        Delay {
            has_instance: AtomicBool::new(true),
            alarm,
            waker: Cell::new(None),
        }
    }

    fn drop_instance(&self) {
        self.has_instance.store(false, Ordering::Relaxed);
    }

    // is 'static required?
    pub fn get_instance(&'static self) -> Option<DelayInstance<'a, A>> {
        if self.has_instance.load(Ordering::Relaxed) {
            self.has_instance.store(true, Ordering::Relaxed);
            Some(DelayInstance::new(self.alarm, self))
        } else {
            None
        }
    }

    fn set_waker(&self, waker: Option<Waker>) {
        self.waker.set(waker);
    }
}

pub struct DelayInstance<'a, A: Alarm<'a>> {
    ns: u32,
    state: State,
    alarm: &'a A,
    delay: &'a Delay<'a, A>,
}

impl<'a, A: Alarm<'a>> DelayInstance<'a, A> {
    fn new(alarm: &'a A, delay: &'a Delay<'a, A>) -> DelayInstance<'a, A> {
        DelayInstance {
            ns: 0,
            state: State::Off,
            alarm,
            delay,
        }
    }
}

impl<'a, A: Alarm<'a>> Future for DelayInstance<'a, A> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.delay.set_waker(Some(cx.waker().clone()));
        match self.state {
            State::Off => {
                debug!("set alarm {:?}", self.alarm.ticks_from_us(self.ns / 1000));
                self.alarm.set_alarm(
                    self.alarm.get_alarm(),
                    self.alarm.ticks_from_us(self.ns / 1000),
                );
                self.state = State::Sleeping;
                Poll::Pending
            }
            State::Sleeping => {
                if !self.alarm.is_armed() {
                    self.state = State::Off;
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

impl<'a, A: Alarm<'a>> DelayNs for DelayInstance<'a, A> {
    async fn delay_ns(&mut self, ns: u32) {
        if let State::Off = self.state {
            self.ns = ns;
            self.await
        } else {
            panic!("already seeping")
        }
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for Delay<'a, A> {
    fn alarm(&self) {
        debug!("alarm");
        let waker = self.waker.take();
        waker.map(|waker| {
            waker.wake();
        });
    }
}

impl<'a, A: Alarm<'a>> Drop for DelayInstance<'a, A> {
    fn drop(&mut self) {
        self.delay.drop_instance();
    }
}
