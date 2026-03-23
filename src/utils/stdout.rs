use std::io::{BufRead, BufReader, Read};
use std::thread;
use std::time::Duration;

pub fn wait_for_output<R: Read + Send + 'static>(
    stdout: R,
    needle: &str,
    verbose: bool,
    verbose_after_catch: bool,
) -> Result<(), anyhow::Error> {
    wait_for_output_timeout(stdout, needle, verbose, verbose_after_catch, None)
}

pub fn wait_for_output_timeout<R: Read + Send + 'static>(
    stdout: R,
    needle: &str,
    verbose: bool,
    verbose_after_catch: bool,
    timeout: Option<Duration>,
) -> Result<(), anyhow::Error> {
    let reader = BufReader::new(stdout);
    let needle = needle.to_string();
    let needle_for_timeout = needle.clone();
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

    match timeout {
        Some(duration) => rx.recv_timeout(duration).map_err(|_| {
            anyhow::anyhow!(
                "Timed out after {:?} waiting for '{}'",
                duration,
                needle_for_timeout
            )
        })?,
        None => rx.recv()?,
    }
}
