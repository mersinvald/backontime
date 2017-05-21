use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::{Command, Stdio};
use std::io::{Write, stderr};
use log::LogLevel;

use global::GLOBAL;

#[derive(Debug)]
pub struct BackupEntity {
    pub path: PathBuf,
    pub recursive: bool,
    pub trigger_changes: u64,
    pub trigger_timer: u64,
    pub exec: String,
    pub last_triggered: SystemTime,
    pub changed: u64
}

impl Default for BackupEntity {
    fn default() -> Self {
        BackupEntity {
            path: PathBuf::from("./"),
            recursive: false,
            trigger_changes: 0,
            trigger_timer: 0,
            exec: String::from("/bin/true"),
            last_triggered: UNIX_EPOCH,
            changed: 0,
        }
    }
}

impl BackupEntity {
    pub fn backup(&mut self) -> Result<()> {
        info!("starting backup on {:?}", self.path.display());
        
        let child = Command::new("bash")
                            .arg("-c")
                            .arg(&self.exec)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("failed to execute command");

        let output = child.wait_with_output()
                            .expect("failed to join child process");

        let log_level = GLOBAL.lock().unwrap().log_level;
        
        let should_print_stdout = !output.status.success() || log_level >= LogLevel::Debug;
        let should_print_stderr = !output.status.success() || log_level >= LogLevel::Trace;

        GLOBAL.lock().unwrap().exec(|| {
            if output.status.success() {
                info!("\"bash -c {:?}\" finished successfully", self.exec)
            } else {
                error!("\"bash -c {:?}\" failed: {}", self.exec, output.status)
            }

            let mut stderr = stderr();

            || -> Result<()> {
                if should_print_stdout {
                    let process_stdout = String::from_utf8_lossy(&output.stdout);
                    writeln!(stderr, "process stdout: ")?;
                    for line in process_stdout.lines() {
                        writeln!(stderr, "1> {}", line)?;
                    }
                }
                
                if should_print_stderr {
                    let process_stderr = String::from_utf8_lossy(&output.stderr);   
                    writeln!(stderr, "process stderr: ")?;
                    for line in process_stderr.lines() {
                        writeln!(stderr, "2> {}", line)?;
                    }
                }

                Ok(())
            }().unwrap();
        });

        self.last_triggered = SystemTime::now();
        self.changed = 0;

        Ok(())
    }
}


error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Notify(::notify::Error);
    }
}