use std::io::{BufRead, BufReader, Read};

pub fn wait_for_output<R: Read>(
    stdout: R,
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
