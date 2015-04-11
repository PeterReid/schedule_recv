
use timer::{oneshot_ms};
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
