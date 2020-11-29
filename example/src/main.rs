use unix_exec_output_catcher::{fork_exec_and_catch, OCatchStrategy};

fn main() {

    // executes "ls" with "-la" as arguments.
    // this is equivalent to running "$ ls -la" in your shell.
    // The line by line output is stored inside the result.
    let res = fork_exec_and_catch("ls", vec!["ls", "-la"], OCatchStrategy::StdSeparately);
    println!("{:#?}", res.unwrap());
}