
use std::collections::BinaryHeap;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::sync::{Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::cmp::{Ordering, min, max};

use time::{SteadyTime, Duration};

struct TimerEvent {
    when: SteadyTime,
    completion_sink: Sender<()>,
    period: Option<u32>,
}
impl Ord for TimerEvent {
    fn cmp(&self, other: &TimerEvent) -> Ordering {
        other.when.cmp(&self.when)
    }
}
impl PartialEq for TimerEvent {
    fn eq(&self, other: &TimerEvent) -> bool {
        let self_ptr: *const TimerEvent = self;
        let other_ptr: *const TimerEvent = other;

        self_ptr == other_ptr
    }
}
impl Eq for TimerEvent {}
impl PartialOrd for TimerEvent {
    fn partial_cmp(&self, other: &TimerEvent) -> Option<Ordering> {
        other.when.partial_cmp(&self.when)
    }
}

struct TimerRequest {
    duration: u32,
    periodic: bool,
    completion_sink: Sender<()>,
}

struct TimerInterface {
    trigger: Arc<Condvar>,
    adder: Sender<TimerRequest>,
}

struct TimerWorker {
    trigger: Arc<Condvar>,
    request_source: Receiver<TimerRequest>,
    schedule: BinaryHeap<TimerEvent>,
}

impl TimerWorker {
    fn new(trigger: Arc<Condvar>, request_source: Receiver<TimerRequest>) -> TimerWorker {
        TimerWorker{
            trigger: trigger,
            request_source: request_source,
            schedule: BinaryHeap::new(),
        }
    }

    fn drain_request_queue(&mut self) {
        while let Ok(request) = self.request_source.try_recv() {
            self.schedule.push(TimerEvent{
                when: SteadyTime::now() + Duration::milliseconds(request.duration as i64),
                period: if request.periodic { Some(request.duration) } else { None },
                completion_sink: request.completion_sink
            });
        }
    }

    fn has_event_now(&self) -> bool {
        if let Some(evt) = self.schedule.peek() {
            evt.when < SteadyTime::now()
        } else {
            false
        }
    }

    fn fire_event(&mut self) {
        if let Some(evt) = self.schedule.pop() {
            match evt.completion_sink.send( () ) {
                Ok( () ) => {
                    if let Some(period) = evt.period.clone() {
                        self.schedule.push(TimerEvent{
                            when: evt.when + Duration::milliseconds(period as i64),
                            period: evt.period,
                            completion_sink: evt.completion_sink,
                        });
                    }
                }
                Err(_) => {
                    // The receiver is no longer waiting for us
                }
            }
        }
    }

    fn ms_until_next_event(&self) -> u32 {
        if let Some(evt) = self.schedule.peek() {
            max(0, min((evt.when - SteadyTime::now()).num_milliseconds(), 100000))  as u32
        } else {
            100000
        }
    }

    fn run(&mut self) {
        let m = Mutex::new(false);
        let mut g = m.lock().unwrap(); // The mutex isn't poisoned, since we just made it

        loop {
            self.drain_request_queue();

            // Fire off as many events as we are supposed to.
            loop {
                if self.has_event_now() {
                    self.fire_event();
                } else {
                    break;
                }
            }

            let wait_millis = self.ms_until_next_event();

            // unwrap() is safe because the mutex will not be poisoned,
            // since we have not shared it with another thread.
            g = self.trigger.wait_timeout_ms(g, wait_millis).unwrap().0;
        }
    }
}

lazy_static! {
    static ref TIMER_INTERFACE  : Mutex<TimerInterface> = {
        let (sender, receiver) = channel();
        let trigger = Arc::new(Condvar::new());
        let trigger2 = trigger.clone();
        thread::spawn(move|| {
            TimerWorker::new(trigger2, receiver).run();
        });

        let interface = TimerInterface {
            trigger: trigger,
            adder: sender
        };

        Mutex::new(interface)
    };
}

fn add_request(duration_ms: u32, periodic: bool) -> Receiver<()> {
    let (sender, receiver) = channel();

    let interface = TIMER_INTERFACE.lock().ok().expect("Failed to acquire the global timer worker");
    interface.adder.send(TimerRequest{
        duration:duration_ms,
        completion_sink:sender,
        periodic: periodic
    }).ok().expect("Failed to send a request to the global timer worker");

    interface.trigger.notify_one();

    receiver
}

pub fn oneshot_ms(ms: u32) -> Receiver<()> {
    add_request(ms, false)
}

pub fn periodic_ms(ms: u32) -> Receiver<()> {
    add_request(ms, true)
}
