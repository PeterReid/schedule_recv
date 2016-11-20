extern crate schedule_recv;

use schedule_recv::periodic_ms;
use std::time::Duration;
use std::thread;

fn main() {
    let tick = periodic_ms(2000);
    thread::sleep(Duration::from_millis(1000));
    let tock = periodic_ms(2000);

    loop {
        tick.recv().unwrap();
        println!("Tick");

        tock.recv().unwrap();
        println!("Tock");
    }
}
