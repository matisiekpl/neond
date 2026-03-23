use std::process::Command;

#[cfg(target_os = "linux")]
pub fn configure_death_signal(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        cmd.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM, 0, 0, 0);
            libc::setpgid(0, 0);
            Ok(())
        });
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn configure_death_signal(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
pub fn configure_death_signal(_cmd: &mut Command) {}
