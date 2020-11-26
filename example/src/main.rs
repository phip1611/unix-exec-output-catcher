use unix_exec_output_catcher::fork_exec_and_catch;

fn main() {
    /*  my lib uses "log"-crate:
        optional
        // trace activates all others
        std::env::set_var("RUST_LOG", "trace");
        // valid values are "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
        // std::env::set_var("RUST_LOG", "trace,info,debug,warn,error");
        env_logger::init();
     */

    // executes "ls" with the "-la" args.
    // this is equivalent to running "$ ls -la" in your shell.
    // The line by line output is stored inside the result.
    let res = fork_exec_and_catch("ls", vec!["ls", "-la"]);
    println!("{:#?}", res.unwrap());
}