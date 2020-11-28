// due to my research: on MacOS AND Linux
//const DEFAULT_PIPE_BUFFER_SIZE: usize = 65_536;
const DEFAULT_PIPE_BUFFER_SIZE: usize = 536;

fn main() {
    // write more then the pipe buffer size to test for
    // dead locks for example
    for i in (0..DEFAULT_PIPE_BUFFER_SIZE) {
        println!( "STDOUT 01/10 @ {:#4}", i);
        eprintln!("STDERR 02/10 @ {:#4}", i);
        println!( "STDOUT 03/10 @ {:#4}", i + 1);
        eprintln!("STDERR 04/10 @ {:#4}", i + 1);
        println!( "STDOUT 05/10 @ {:#4}", i + 3);
        println!( "STDOUT 06/10 @ {:#4}", i + 3);
        println!( "STDOUT 07/10 @ {:#4}", i + 3);
        eprintln!("STDERR 08/10 @ {:#4}", i + 1);
        eprintln!("STDERR 09/10 @ {:#4}", i + 1);
        eprintln!("STDERR 10/10 @ {:#4}", i + 1);
    }
}