/// This crate exposes functionality to create receivers that
/// receive notifications after a specified period of time or at
/// a specified frequency.
///
/// # Examples
///
/// At its simplest, oneshot_ms can be used to put the thread to
/// sleep. Unlike with std::thread::sleep, this could be used with
/// Select to be waiting for one of several Receivers to fire.
///
/// ```
/// # use schedule_recv::oneshot_ms;
/// # fn sleep_equivalent() {
/// let timer = oneshot_ms(1500);
/// timer.recv().unwrap();
/// println!("1.5 seconds have elapsed.");
/// # }
/// ```
///
/// Periodic Receivers can be created using periodic_ms.
///
/// ```
/// # use schedule_recv::periodic_ms;
/// # use std::thread;
/// # fn tick_tock() {
/// let tick = periodic_ms(2000);
/// thread::sleep_ms(1000);
/// let tock = periodic_ms(2000);
///
/// loop {
///     tick.recv().unwrap();
///     println!("Tick");
///     tock.recv().unwrap();
///     println!("Tock");
/// }
/// # }
/// ```


#[macro_use] extern crate lazy_static;
extern crate time;

mod scheduler;

#[cfg(test)] mod test;

pub use scheduler::{oneshot_ms, periodic_ms};
