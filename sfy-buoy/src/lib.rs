#![feature(test)]
#![feature(inline_const)]
#![feature(const_option_ext)]
#![feature(result_option_inspect)]
#![cfg_attr(not(feature = "host-tests"), no_std)]

#[cfg(test)]
extern crate test;

#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};

// we use this for defs of sinf etc.
extern crate cmsis_dsp;

use ambiq_hal::{delay::FlashDelay, rtc::Rtc};
use chrono::NaiveDateTime;
use core::cell::RefCell;
use core::fmt::Debug;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicI32, Ordering};
use cortex_m::interrupt::{free, Mutex};
use embedded_hal::blocking::{
    delay::DelayMs,
    i2c::{Read, Write, WriteRead},
};

pub mod axl;
pub mod fir;
pub mod log;
pub mod note;
#[cfg(feature = "storage")]
pub mod storage;
pub mod waves;

use axl::AxlPacket;

/// These queues are filled up by the IMU interrupt in read batches of time-series. It is then consumed
/// the main thread and first drained to the SD storage (if enabled), and then queued for the notecard.
#[cfg(feature = "storage")]
pub static mut STORAGEQ: heapless::spsc::Queue<AxlPacket, 32> = heapless::spsc::Queue::new();

pub static mut NOTEQ: heapless::spsc::Queue<AxlPacket, 32> = heapless::spsc::Queue::new();

/// The STATE contains the Real-Time-Clock which needs to be shared, as well as up-to-date
/// longitude and latitude.
pub static STATE: Mutex<RefCell<Option<SharedState>>> = Mutex::new(RefCell::new(None));

pub static COUNT: AtomicI32 = AtomicI32::new(0);
defmt::timestamp!("{=i32}", COUNT.load(Ordering::Relaxed));

pub struct SharedState {
    pub rtc: Rtc,
    pub position_time: u32,
    pub lon: f64,
    pub lat: f64,
}

pub trait State {
    fn now(&self) -> NaiveDateTime;
}

impl State for Mutex<RefCell<Option<SharedState>>> {
    fn now(&self) -> NaiveDateTime {
        free(|cs| {
            let state = self.borrow(cs).borrow();
            let state = state.as_ref().unwrap();

            state.rtc.now()
        })
    }
}

#[derive(Clone)]
pub enum LocationState {
    Trying(i64),
    Retrieved(i64),
}

#[derive(Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub position_time: u32,
    pub time: u32,

    pub state: LocationState,
}

impl Location {
    pub fn new() -> Location {
        Location {
            lat: 0.0,
            lon: 0.0,
            position_time: 0,
            time: 0,
            state: LocationState::Trying(-999),
        }
    }

    pub fn check_retrieve<T: Read + Write>(
        &mut self,
        state: &Mutex<RefCell<Option<SharedState>>>,
        delay: &mut impl DelayMs<u16>,
        note: &mut note::Notecarrier<T>,
    ) -> Result<(), notecard::NoteError> {
        use notecard::card::res::{Location, Time};
        use LocationState::*;

        const LOCATION_DIFF: i64 = 1 * 60_000; // ms

        let now = state.now().timestamp_millis();
        defmt::trace!("now: {}", now);

        match self.state {
            Retrieved(t) | Trying(t) if (now - t) > LOCATION_DIFF => {
                let gps = note.card().location(delay)?.wait(delay)?;
                let tm = note.card().time(delay)?.wait(delay);

                info!("Location: {:?}, Time: {:?}", gps, tm);
                if let (
                    Location {
                        lat: Some(lat),
                        lon: Some(lon),
                        time: Some(position_time),
                        ..
                    },
                    Ok(Time {
                        time: Some(time), ..
                    }),
                ) = (gps, tm)
                {
                    info!("got time and location, setting RTC.");

                    self.lat = lat;
                    self.lon = lon;
                    self.position_time = position_time;
                    self.time = time;

                    free(|cs| {
                        let mut state = state.borrow(cs).borrow_mut();
                        let state: &mut _ = state.deref_mut().as_mut().unwrap();

                        state.rtc.set(NaiveDateTime::from_timestamp(time as i64, 0));
                        state.position_time = position_time;
                        state.lat = lat;
                        state.lon = lon;

                        self.state = Retrieved(state.rtc.now().timestamp_millis());
                    });
                } else {
                    self.state = Trying(now);
                }
            }
            _ => (),
        }

        Ok(())
    }
}

pub struct Imu<E: Debug + defmt::Format, I: Write<Error = E> + WriteRead<Error = E>> {
    pub queue: heapless::spsc::Producer<'static, AxlPacket, 32>,
    waves: waves::Waves<I>,
}

impl<E: Debug + defmt::Format, I: Write<Error = E> + WriteRead<Error = E>> Imu<E, I> {
    pub fn new(
        waves: waves::Waves<I>,
        queue: heapless::spsc::Producer<'static, AxlPacket, 32>,
    ) -> Imu<E, I> {
        Imu { queue, waves }
    }

    pub fn check_retrieve(
        &mut self,
        now: i64,
        position_time: u32,
        lon: f64,
        lat: f64,
    ) -> Result<(), waves::ImuError<E>> {
        trace!("Polling IMU.. (now: {})", now,);

        self.waves.read_and_filter()?;

        if self.waves.is_full() {
            trace!("waves buffer is full, pushing to queue..");
            let pck = self.waves.take_buf(now, position_time, lon, lat)?;

            self.queue
                .enqueue(pck)
                .inspect_err(|pck| {
                    error!("queue is full, discarding data: {}", pck.data.len());

                    log::log("Queue is full: discarding package.");
                })
                .ok();
        }

        Ok(())
    }

    pub fn reset(&mut self, now: i64, position_time: u32, lon: f64, lat: f64) -> Result<(), waves::ImuError<E>> {
        self.waves.reset()?;
        self.waves.take_buf(now, position_time, lon, lat)?; // buf is empty, this sets time and offset.
        self.waves.enable_fifo(&mut FlashDelay)?;

        Ok(())
    }
}
