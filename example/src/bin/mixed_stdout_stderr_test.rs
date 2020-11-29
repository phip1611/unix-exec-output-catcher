// due to my research: on MacOS AND Linux
const DEFAULT_PIPE_BUFFER_SIZE: usize = 65_536;
const ITERATIONS: usize = 10;

fn main() {
    // use this to check if my lib captures the output in right order
    for i in 0..ITERATIONS {
        println!( "STDOUT 01/10 @ {:#4}", i);
        eprintln!("STDERR 02/10 @ {:#4}", i);
        println!( "STDOUT 03/10 @ {:#4}", i);
        eprintln!("STDERR 04/10 @ {:#4}", i);
        println!( "STDOUT 05/10 @ {:#4}", i);
        println!( "STDOUT 06/10 @ {:#4}", i);
        println!( "STDOUT 07/10 @ {:#4}", i);
        eprintln!("STDERR 08/10 @ {:#4}", i);
        eprintln!("STDERR 09/10 @ {:#4}", i);
        eprintln!("STDERR 10/10 @ {:#4}", i);
    }
}