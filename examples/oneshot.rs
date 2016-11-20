extern crate schedule_recv;

use schedule_recv::oneshot_ms;

fn main() {
    let timer = oneshot_ms(1500);
    timer.recv().unwrap();

    println!("1.5 seconds have elapsed.");
}
