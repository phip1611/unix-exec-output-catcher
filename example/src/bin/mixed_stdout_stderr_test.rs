use std::thread::sleep;
use std::time::Duration;

const ITERATIONS: usize = 2000;
// the less this value is the more likely that the total captured
// output of stdout and stderr combined is out of order
const DELAY_US: u64 = 150;

/// This binary can be used to check the output catching of my lib.
/// It produces a mixture of STDOUT and STDERR lines in a defined
/// order. The bin `run_mixed_stdout_stderr_test` is a support
/// bin that executes this binary inside the library.
/// This way I can make tests for the correct output order.
fn main() {
    for i in 0..ITERATIONS {
        println!( "STDOUT 01/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        eprintln!("STDERR 02/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        println!( "STDOUT 03/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        eprintln!("STDERR 04/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        println!( "STDOUT 05/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        println!( "STDOUT 06/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        println!( "STDOUT 07/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        eprintln!("STDERR 08/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        eprintln!("STDERR 09/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
        eprintln!("STDERR 10/10 @ {:#4}", i);
        sleep(Duration::from_micros(DELAY_US));
    }
}