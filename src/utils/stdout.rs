use std::io::{BufRead, BufReader, Read};
use std::thread;

pub fn wait_for_output<R: Read + Send + 'static>(
    stdout: R,
    needle: &str,
    verbose: bool,
    verbose_after_catch: bool,
) -> Result<(), anyhow::Error> {
    let reader = BufReader::new(stdout);
    let needle = needle.to_string();
    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let mut caught = false;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            if verbose && !caught {
                println!("{}", line);
            }
            if line.contains(&needle) && !caught {
                caught = true;
                let _ = tx.send(Ok(()));
            }
            if verbose_after_catch && caught {
                println!("{}", line);
            }
        }
        if !caught {
            let _ = tx.send(Err(anyhow::anyhow!(
                "Process exited before printing '{}'",
                needle
            )));
        }
    });

    rx.recv()?
}
