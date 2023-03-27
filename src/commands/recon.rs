use sysinfo::{CpuExt, Pid, ProcessExt, System, SystemExt, UserExt};
use whoami::*;

pub fn whoami() -> String {
    // Get system information
    let mut sys = System::new_all();
    sys.refresh_all();

    let kernel_version = format!(
        " - Kernel Version: ({})",
        sys.kernel_version().unwrap_or_default()
    );
    let user = format!(r#"\{} ({})"#, username(), realname());

    let mut cpu_info = String::new();
    if let Some(cpu) = sys.cpus().iter().next() {
        cpu_info = format!("CPU: {} {} mHz", cpu.brand(), cpu.frequency());
    }

    format!(
        "```java\n\
        Hostname: {:?}\n\
        OS Version: {:?}\n\
        CPU Info: {:?}```",
        sys.host_name().unwrap_or_default() + &user,
        sys.long_os_version().unwrap_or_default() + &kernel_version,
        cpu_info
    )
}

pub fn userlist() -> String {
    struct User {
        username: String,
        id: String,
        groups: Vec<String>,
    }

    impl ToString for User {
        fn to_string(&self) -> String {
            format!(
                "```ini\n\
                Username: [{}], ID: [{}], Groups: [{}]```",
                self.username,
                self.id,
                self.groups.join(", ")
            )
        }
    }
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut user_structs = Vec::new();

    for user in sys.users() {
        let user_struct = User {
            username: user.name().to_string(),
            id: user.id().to_string(),
            groups: user.groups().iter().map(|g| g.to_string()).collect(),
        };
        user_structs.push(user_struct);
    }

    user_structs
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<String>>()
        .join("")
}

pub fn tasklist() -> String {
    struct Task {
        pid: Pid,
        name: String,
        memory: u64,
        parent: Option<Pid>,
        user_name: Option<String>,
    }

    impl ToString for Task {
        fn to_string(&self) -> String {
            format!(
                "```ini\n\
                PID: [{}], Name: [{}], Memory: [{} bytes], Parent: [{}], User Name: [{}]```\n",
                self.pid,
                self.name,
                self.memory,
                self.parent.unwrap_or(Pid::from(0)),
                self.user_name.as_ref().unwrap_or(&String::from("N/A")),
            )
        }
    }

    let mut sys = System::new_all();
    sys.refresh_all();
    let mut task_structs = Vec::new();

    for (pid, process) in sys.processes() {
        let user_name = if let Some(user_id) = process.user_id() {
            if let Some(user) = sys.get_user_by_id(user_id) {
                Some(user.name().to_string())
            } else {
                None
            }
        } else {
            None
        };

        let task = Task {
            pid: *pid,
            name: process.name().to_string(),
            memory: process.memory(),
            parent: process.parent(),
            user_name,
        };
        task_structs.push(task);
    }

    task_structs
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("")
}
