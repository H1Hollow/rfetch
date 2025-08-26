use libc::{
    AF_INET, c_char, freeifaddrs, gethostname, getifaddrs, ifaddrs, sockaddr_in, statvfs, sysinfo,
    utsname,
};

use raw_cpuid::CpuId;

use std::{
    env,
    ffi::{CStr, CString},
    fs::{self, File},
    io::{BufRead, BufReader},
    net::Ipv4Addr,
    ptr,
    time::Duration,
};

mod helpers;
pub use helpers::*;

static RESET_CODE: &str = "\x1b[0m";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn help_menu() {
    println!(
        "rfetch {VERSION}

USAGE:
    rfetch [OPTIONS]

OPTIONS (optional):
    --config <FILE>     path to text file containing ascii art
    --spacing <N>       spaces before ASCII art (0–255, default=3)
    --color <ANSI>      (e.g. 36, 1;36, 38;5;205)
    -h, --help          print help
    -v, --version       print version"
    );
}

pub fn memory_usage() -> String {
    let values = read_meminfo_fields(&["MemTotal:", "MemAvailable:"]);
    let total = values.get(0).copied().unwrap_or(0);
    let available = values.get(1).copied().unwrap_or(0);

    let used = total.saturating_sub(available);
    format!(
        "Memory: {}/{}",
        format_bytes(used * 1024),
        format_bytes(total * 1024)
    )
}

pub fn swap_usage() -> String {
    let values = read_meminfo_fields(&["SwapTotal:", "SwapFree:"]);
    let total = values.get(0).copied().unwrap_or(0);
    let free = values.get(1).copied().unwrap_or(0);

    let used = total.saturating_sub(free);
    format!(
        "Swap: {}/{}",
        format_bytes(used * 1024),
        format_bytes(total * 1024)
    )
}

fn get_root_disk_usage() -> String {
    let path = "/";
    let c_path = CString::new(path).unwrap();
    let mut stat: statvfs = unsafe { std::mem::zeroed() };

    let ret = unsafe { statvfs(c_path.as_ptr() as *const c_char, &mut stat) };
    if ret != 0 {
        return "Disk usage: unknown".to_string();
    }

    let total = stat.f_blocks * stat.f_frsize as u64;
    let free = stat.f_bfree * stat.f_frsize as u64;
    let used = total - free;

    // convert bytes to gb
    let total_gb = total / 1024 / 1024 / 1024;
    let used_gb = used / 1024 / 1024 / 1024;

    format!("Disk: {}GB/{}GB used (/)", used_gb, total_gb)
}

fn read_os_release(path: &str) -> (String, String) {
    let mut os_name = "Unknown OS".to_string();
    let mut ansi_color = "0;37".to_string(); // default white
    let mut found_name = false;
    let mut found_color = false;

    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if !found_name && line.starts_with("PRETTY_NAME=") {
                os_name = line["PRETTY_NAME=".len()..].trim_matches('"').to_string();
                found_name = true;
            } else if !found_color && line.starts_with("ANSI_COLOR=") {
                ansi_color = line["ANSI_COLOR=".len()..].trim_matches('"').to_string();
                found_color = true;
            }
            if found_name && found_color {
                break; // stop early once we have both
            }
        }
    }

    (os_name, ansi_color)
}

fn color_ascii_art(ascii_art: &str, color_code: &str, spacing: u8) -> Vec<String> {
    let prefix = " ".repeat(spacing as usize);
    ascii_art
        .lines()
        .map(|line| format!("{}{}{}{}", prefix, color_code, line, RESET_CODE))
        .collect()
}

fn get_user() -> String {
    let username = env::var("USER").unwrap_or_else(|_| "unknown".into());
    let hostname = unsafe {
        let mut buf = [0u8; 256];
        if gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) == 0 {
            CStr::from_ptr(buf.as_ptr() as *const i8)
                .to_string_lossy()
                .split('.')
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else {
            "unknown".into()
        }
    };

    // Append product name
    let model = fs::read_to_string("/sys/class/dmi/id/product_name")
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();

    format!("{}@{}@{}", username, hostname, model)
}

fn get_cpu() -> String {
    CpuId::new()
        .get_processor_brand_string()
        .map(|b| b.as_str().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string())
}

fn get_kernel() -> String {
    unsafe {
        let mut uts: utsname = std::mem::zeroed();
        if libc::uname(&mut uts) == 0 {
            let sysname = CStr::from_ptr(uts.sysname.as_ptr()).to_string_lossy();
            let release = CStr::from_ptr(uts.release.as_ptr()).to_string_lossy();
            format!("KERNEL: {} {}", sysname, release)
        } else {
            "KERNEL: unknown".to_string()
        }
    }
}

fn get_local_ip() -> String {
    unsafe {
        let mut ifap: *mut ifaddrs = ptr::null_mut();
        if getifaddrs(&mut ifap) != 0 {
            return "Local IP: unknown".to_string();
        }

        let mut ptr_ifap = ifap;
        while !ptr_ifap.is_null() {
            let ifa = &*ptr_ifap;
            if !ifa.ifa_addr.is_null() && (*ifa.ifa_addr).sa_family as i32 == AF_INET {
                let sa = &*(ifa.ifa_addr as *const sockaddr_in);
                let ip = Ipv4Addr::from(u32::from_be(sa.sin_addr.s_addr));
                if ip != Ipv4Addr::new(127, 0, 0, 1) {
                    freeifaddrs(ifap);
                    return format!("Local IP: {}", ip);
                }
            }
            ptr_ifap = ifa.ifa_next;
        }

        freeifaddrs(ifap);
        "Local IP: unknown".to_string()
    }
}

fn get_uptime() -> String {
    unsafe {
        let mut info: sysinfo = std::mem::zeroed();
        if libc::sysinfo(&mut info) == 0 {
            let duration = Duration::from_secs(info.uptime as u64);
            let hours = duration.as_secs() / 3600;
            let minutes = (duration.as_secs() % 3600) / 60;
            format!("Uptime: {} hours, {} mins", hours, minutes)
        } else {
            "Uptime: unknown".to_string()
        }
    }
}

fn print_stuff(ascii_lines: &[String], sys_info: &[String], spacing: u8) {
    let max_ascii_len = ascii_lines
        .iter()
        .map(|line| line.len() - spacing as usize)
        .max()
        .unwrap_or(0);
    let offset = max_ascii_len + 5 + spacing as usize;

    let max_lines = ascii_lines.len().max(sys_info.len());
    let empty: &str = "";
    for i in 0..max_lines {
        let art_line: &str = if i < ascii_lines.len() {
            &ascii_lines[i]
        } else {
            empty
        };
        let info_line: &str = if i < sys_info.len() { &sys_info[i] } else { "" };
        let spaces = " ".repeat(offset.saturating_sub(art_line.len()));
        println!("{}{}{}", art_line, spaces, info_line);
    }
}

// main

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        help_menu();
        return;
    } else if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("rfetch {}", VERSION);
        return;
    }

    let mut ascii_art = r#"        #####
       #######
       ##O#O##
       #######
     ###########
    #############
   ###############
   ################
  #################
  #################
#####################
#####################
  #################
"#
    .to_string();
    let mut spacing: u8 = 3;

    let (os_name, mut ansi_color) = if std::path::Path::new("/etc/os-release").exists() {
        read_os_release("/etc/os-release")
    } else if std::path::Path::new("/usr/lib/os-release").exists() {
        read_os_release("/usr/lib/os-release")
    } else {
        ("Unknown OS".to_string(), "0;37".to_string())
    };

    // parse args manually
    let mut iter = args.iter().skip(1); // skip program name
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                if let Some(path) = iter.next() {
                    ascii_art = read_file_trim(path);
                }
            }
            "--spacing" => {
                if let Some(val) = iter.next() {
                    if let Ok(num) = val.parse::<u8>() {
                        spacing = num;
                    }
                }
            }
            "--color" => {
                if let Some(c) = iter.next() {
                    ansi_color = c.clone();
                }
            }
            _ => {}
        }
    }

    let color_code = format!("\x1b[{}m", ansi_color);
    let colored_art_lines = color_ascii_art(&ascii_art, &color_code, spacing);

    let separator = make_separator(get_user().len(), &color_code);

    let sys_info = vec![
        get_user(),
        get_uptime(),
        separator.clone(),
        format!("OS: {}", os_name),
        format!("CPU: {}", get_cpu()),
        get_kernel(),
        get_root_disk_usage(),
        memory_usage(),
        swap_usage(),
        format!("Terminal: {}", evod("TERM", "unknown")),
        format!("Shell: {}", evod("SHELL", "unknown")),
        format!("WM: {}", evod("XDG_CURRENT_DESKTOP", "unknown")),
        get_local_ip(),
    ];

    print_stuff(&colored_art_lines, &sys_info, spacing);
}
