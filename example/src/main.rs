use unix_exec_output_catcher::fork_exec_and_catch;

// #[macro_use]
// extern crate log;

fn main() {
    // trace activates all others
    std::env::set_var("RUST_LOG", "trace");
    // valid values are "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
    // std::env::set_var("RUST_LOG", "trace,info,debug,warn,error");
    env_logger::init();

    /*trace!("Test Trace Level");
    debug!("Test Debug Level");
    warn!("Test Warn Level");
    error!("Test Error Level");
    info!("Test Info Level");*/

    // let res = fork_exec_and_catch("ls", vec!["ls", "-la", "--color=always"]);
    //let res = fork_exec_and_catch("ls", vec!["ls", "-la"]);
    let res = fork_exec_and_catch("cat", vec!["cat", "/dev/random"]);
    // let res = fork_exec_and_catch("pwd", vec!["pwd"]);

    //println!("{:#?}", res.unwrap());
    println!("{:#?}", res.unwrap().stdcombined_lines());
}