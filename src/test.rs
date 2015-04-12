
use scheduler::{oneshot_ms, periodic_ms};
use time::SteadyTime;

struct ElapsedChecker {
    start: SteadyTime
}

impl ElapsedChecker {
    fn new() -> ElapsedChecker {
        ElapsedChecker {
            start: SteadyTime::now()
        }
    }

    fn check(&self, expected_ms: i64) {
        let actual_elapsed_ms = (SteadyTime::now() - self.start).num_milliseconds();
        assert!(actual_elapsed_ms-100 < expected_ms, "Elapsed too late: {}ms instead of {}ms", actual_elapsed_ms, expected_ms);
        assert!(actual_elapsed_ms+100 > expected_ms, "Elapsed too soon: {}ms instead of {}ms", actual_elapsed_ms, expected_ms);
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

    for i in (1..10) {
        p.recv().unwrap();
        checker.check(i*200);
    }
}
