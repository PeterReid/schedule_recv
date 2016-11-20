This module exposes functionality to create receivers that
receive notifications after a specified period of time or at
a specified frequency.

[![Build status](https://api.travis-ci.org/PeterReid/schedule_recv.png)](https://travis-ci.org/PeterReid/schedule_recv)

[Documentation](https://PeterReid.github.io/schedule_recv)

# Examples

At its simplest, oneshot_ms can be used to put the thread to
sleep. Unlike with std::thread::sleep, this could be used with
Select to be waiting for one of several Receivers to fire.

```rust
use schedule_recv::oneshot_ms;

let timer = oneshot_ms(1500);
timer.recv().unwrap();
println!("1.5 seconds have elapsed.");
```

Periodic Receivers can be created using `periodic_ms`.

```rust
use schedule_recv::periodic_ms;

let tick = periodic_ms(2000);
thread::sleep_ms(1000);
let tock = periodic_ms(2000);
loop {
    tick.recv().unwrap();
    println!("Tick");
    tock.recv().unwrap();
    println!("Tock");
}
```
