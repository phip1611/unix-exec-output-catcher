use unix_exec_output_catcher::{fork_exec_and_catch, OCatchStrategy};

fn main() {
    // executes "ls" with "-la" as arguments.
    // this is equivalent to running "$ ls -la" in your shell.
    // The line by line output is stored inside the result.
    let res_1 = fork_exec_and_catch(
        "ls",
        vec!["ls", "-la"],
        OCatchStrategy::StdSeparately
    );
    println!("OCatchStrategy::StdSeparately:");
    println!("{:#?}", res_1.unwrap());

    // Using the other strategy. See `OCatchStrategy` to get more detail.
    let res_2 = fork_exec_and_catch(
        "ls",
        vec!["ls", "-la"],
        OCatchStrategy::StdCombined
    );
    println!("OCatchStrategy::StdCombined:");
    println!("{:#?}", res_2.unwrap());
}