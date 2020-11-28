use unix_exec_output_catcher::fork_exec_and_catch;
use std::fmt::Debug;

fn is_sorted<T>(data: &[T]) -> bool
    where T: Ord,
{
    assert_eq!(data.len() % 10, 0);
    let window_count = data.len() / 10;
    for i in 0..window_count {
        let x = i * 10;
        let non_overlapping_window = &data[x..x+10];
        let sorted = non_overlapping_window.windows(2).all((|wi| {
            wi[0] <= wi[1]
        }));
        if !sorted {
            return false;
        }
    }
    true
}

fn main() {
    let res = fork_exec_and_catch(
        "./target/debug/mixed_stdout_stderr_test",
        vec!["mixed_stdout_stderr_test"])
        .unwrap();
    /*let res = fork_exec_and_catch(
        "pwd",
        vec!["pwd"])
        .unwrap();*/

    let all_lines = res.stdcombined_lines()
        .into_iter()
        .map(|s| s.replace("STDERR ", ""))
        .map(|s| s.replace("STDOUT ", ""))
        .map(|s| s.split(" @")
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
        )
        .map(|v| v[0].to_string())
        .map(|s| s.split("/")
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
        )
        .map(|v| v[0].to_string())
        .collect::<Vec<String>>();

    println!("Is output in right order?");


    let is_sorted = is_sorted(&all_lines);
    if is_sorted {
        println!("YES")
    } else {
        eprintln!("NO! TEST FAILED!")
    }
    // println!("{:#?}", all_lines);
}