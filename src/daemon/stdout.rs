use std::io::{BufRead, BufReader};
use std::process::ChildStdout;

pub fn wait_for_output(
    stdout: ChildStdout,
    needle: &str,
    verbose: bool,
) -> Result<(), anyhow::Error> {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let line = line?;
        if verbose {
            println!("{}", line);
        }
        if line.contains(needle) {
            return Ok(());
        }
    }
    anyhow::bail!("Process exited before printing '{}'", needle);
}
