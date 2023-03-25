use sysinfo::{System, SystemExt};


pub fn whoami() -> String {
    // Get system information
    let mut sys = System::new_all();
    sys.refresh_all();
    format!(
        "```Hostname: {:?}\n\
        OS Version: {:?}\n\
        Kernel version: {:?}\n\
        CPU Cores: {:?}```",
        sys.host_name().unwrap_or_default(),
        sys.long_os_version().unwrap_or_default(),
        sys.kernel_version().unwrap_or_default(),
        sys.physical_core_count().unwrap_or_default()
    )
}