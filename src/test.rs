
use std::time::{Instant, Duration};

use scheduler::{oneshot_ms, periodic_ms};

struct ElapsedChecker {
    start: Instant
}

impl ElapsedChecker {
    fn new() -> ElapsedChecker {
        ElapsedChecker {
            start: Instant::now()
        }
    }

    fn check(&self, expected_ms: u64) {
        let actual_elapsed = self.start.elapsed();
        let expected = Duration::from_millis(expected_ms);
        assert!(actual_elapsed > Duration::from_millis(100), "Less than 100ms elapsed");
        assert!(actual_elapsed-Duration::from_millis(100) < expected, "Elapsed too late: {:?} instead of {:?}", actual_elapsed, expected);
        assert!(actual_elapsed+Duration::from_millis(100) > expected, "Elapsed too soon: {:?} instead of {:?}", actual_elapsed, expected);
    }
}

#[test]
fn simple_wait() {
    let checker = ElapsedChecker::new();
    let timer = oneshot_ms(1400);
    timer.recv().unwrap();
    checker.check(1400);
}

#[test]
fn several_concurrent_waits() {
    let checker = ElapsedChecker::new();
    let medium = oneshot_ms(1400);
    let short = oneshot_ms(300);
    let long = oneshot_ms(2000);
    short.recv().unwrap();
    checker.check(300);

    medium.recv().unwrap();
    checker.check(1400);

    long.recv().unwrap();
    checker.check(2000);
}

#[test]
fn several_concurrent_waits_misordered() {
    let checker = ElapsedChecker::new();
    let medium = oneshot_ms(1400);
    let short = oneshot_ms(300);
    let long = oneshot_ms(2000);

    // We wait for the last timer before we check the others,
    // so they should all recv immediately after the first one.
    long.recv().unwrap();
    checker.check(2000);

    short.recv().unwrap();
    checker.check(2000);

    medium.recv().unwrap();
    checker.check(2000);
}


#[test]
fn simple_periodic() {
    let checker = ElapsedChecker::new();
    let p = periodic_ms(200);

    for i in 1..10 {
        p.recv().unwrap();
        checker.check(i*200);
    }
}
