
use std::collections::BinaryHeap;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::sync::{Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::cmp::{Ordering};
use std::time::{Instant, Duration};

struct ScheduledEvent {
    when: Instant,
    completion_sink: Sender<()>,
    period: Option<Duration>,
}
impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &ScheduledEvent) -> bool {
        let self_ptr: *const ScheduledEvent = self;
        let other_ptr: *const ScheduledEvent = other;

        self_ptr == other_ptr
    }
}
impl Eq for ScheduledEvent {}
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &ScheduledEvent) -> Ordering {
        other.when.cmp(&self.when)
    }
}
impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &ScheduledEvent) -> Option<Ordering> {
        other.when.partial_cmp(&self.when)
    }
}

struct SchedulingInterface {
    trigger: Arc<Condvar>,
    adder: Sender<ScheduledEvent>,
}

struct ScheduleWorker {
    trigger: Arc<Condvar>,
    request_source: Receiver<ScheduledEvent>,
    schedule: BinaryHeap<ScheduledEvent>,
}

impl ScheduleWorker {
    fn new(trigger: Arc<Condvar>, request_source: Receiver<ScheduledEvent>) -> ScheduleWorker {
        ScheduleWorker{
            trigger: trigger,
            request_source: request_source,
            schedule: BinaryHeap::new(),
        }
    }

    fn drain_request_queue(&mut self) {
        while let Ok(request) = self.request_source.try_recv() {
            self.schedule.push(request);
        }
    }

    fn has_event_now(&self) -> bool {
        if let Some(evt) = self.schedule.peek() {
            evt.when <= Instant::now()
        } else {
            false
        }
    }

    fn fire_event(&mut self) {
        if let Some(evt) = self.schedule.pop() {
            if evt.completion_sink.send( () ).is_ok() {
                if let Some(period) = evt.period {
                    self.schedule.push(ScheduledEvent{
                        when: evt.when + period,
                        period: evt.period,
                        completion_sink: evt.completion_sink,
                    });
                }
            }
        }
    }

    fn dur_until_next_event(&self) -> Option<Duration> {
        self.schedule.peek().map(|evt| {
            let now = Instant::now();
            if evt.when <= now {
                Duration::from_secs(0)
            } else {
                evt.when.duration_since(now)
            }
        })
    }

    fn run(&mut self) {
        let m = Mutex::new( () );
        let mut g = m.lock().unwrap(); // The mutex isn't poisoned, since we just made it

        loop {
            self.drain_request_queue();

            // Fire off as many events as we are supposed to.
            while self.has_event_now() {
                self.fire_event();
            }

            let wait_duration = self.dur_until_next_event();

            // unwrap() is safe because the mutex will not be poisoned,
            // since we have not shared it with another thread.
            g = if let Some(wait_duration) = wait_duration {
                self.trigger.wait_timeout(g, wait_duration).unwrap().0
            } else {
                self.trigger.wait(g).unwrap()
            };
        }
    }
}

lazy_static! {
    static ref SCHEDULER_INTERFACE : Mutex<SchedulingInterface> = {
        let (sender, receiver) = channel();
        let trigger = Arc::new(Condvar::new());
        let trigger2 = trigger.clone();
        thread::spawn(move|| {
            ScheduleWorker::new(trigger2, receiver).run();
        });

        let interface = SchedulingInterface {
            trigger: trigger,
            adder: sender
        };

        Mutex::new(interface)
    };
}

fn add_request(duration: Duration, period: Option<Duration>) -> Receiver<()> {
    let (sender, receiver) = channel();

    let interface = SCHEDULER_INTERFACE.lock().expect("Failed to acquire the global scheduling worker");
    interface.adder.send(ScheduledEvent {
        when: Instant::now() + duration,
        completion_sink: sender,
        period: period
    }).expect("Failed to send a request to the global scheduling worker");

    interface.trigger.notify_one();

    receiver
}

/// Starts a timer which after `ms` milliseconds will issue a **single** `.send(())` on the other side of the
/// returned `Reciever<()>`.
pub fn oneshot_ms(ms: u32) -> Receiver<()> {
    oneshot(Duration::from_millis(ms as u64))
}

/// Starts a timer which, **every** `ms` milliseconds, will issue `.send(())` on the other side of the
/// returned `Reciever<()>`.
pub fn periodic_ms(ms: u32) -> Receiver<()> {
    periodic(Duration::from_millis(ms as u64))
}

/// Starts a timer which after `duration` will issue a **single** `.send(())` on the other side of the
/// returned `Receiver<()>`.
pub fn oneshot(duration: Duration) -> Receiver<()> {
    add_request(duration, None)
}

/// Starts a timer which, every `duration`, will issue a **single** `.send(())` on the other side of the
/// returned `Receiver<()>`.
pub fn periodic(duration: Duration) -> Receiver<()> {
    add_request(duration, Some(duration))
}

/// Starts a timer which, **every** `duration` **after** an initial delay of `start`, will issue `.send(())`
/// on the other side of the returned `Reciever<()>`.
pub fn periodic_after(period: Duration, start: Duration) -> Receiver<()> {
    add_request(start, Some(period))
}
