use crate::CatchAll;
use std::collections::HashMap;
use std::process::{Command, Output};

pub fn check_for_iproute2() -> Result<(), String> {
    const TOOLS: [&str; 4] = ["tc", "ss", "ifstat", "ip"];
    for tool in &TOOLS {
        if let Err(e) = std::process::Command::new(tool).output() {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(format!("Missing program: {}\nIs iproute2 installed?", tool));
            }
        }
    }
    Ok(())
}

// run macro
#[macro_export]
macro_rules! run {
    ($($arg:tt)*) => {
        crate::utils::run(format!($($arg)*))
    }
}

pub fn run(v: String) -> CatchAll<Output> {
    // log all cmds
    // dbg!(&v);
    let cmd = v.clone();
    let mut cmd = cmd.split_whitespace();
    let output = Command::new(cmd.next().expect("Tried to run an empty command"))
        .args(cmd.collect::<Vec<&str>>())
        .output()?;
    if !output.stderr.is_empty() {
        eprintln!(
            "Error while running cmd: {:?}\nerr: {}",
            v,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output)
}

#[test]
fn tifstat() {
    dbg!(ifstat());
}
// ifstat
pub fn ifstat() -> CatchAll<Vec<Interface>> {
    let output = run!("ifstat")?;
    let output = String::from_utf8(output.stdout)?;

    let interfaces = output
        .lines()
        .skip(3)
        .step_by(2)
        .filter_map(|l| l.split_whitespace().next())
        .map(|name| Interface {
            name: name.to_string(),
            // A disadvantage of using ifstat is that we cant tell if the interface is up or not
            // Workaround: always assume its down
            status: Status::Down,
        })
        .collect();

    Ok(interfaces)
}

#[derive(PartialEq, Eq, Debug)]
pub struct Interface {
    pub name: String,
    status: Status,
}

impl Interface {
    pub fn is_up(&self) -> bool {
        self.status == Status::Up
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Status {
    Up,
    Down,
}

// ss
#[test]
fn tss() {
    dbg!(ss());
}

pub fn ss() -> CatchAll<HashMap<String, Vec<Connection>>> {
    let raw_net_table = run!("ss -n -t -p  state established")?;
    let raw_net_table = String::from_utf8(raw_net_table.stdout)?;

    let mut net_table = HashMap::new();

    let mut parse = |row: &str| -> Option<()> {
        let mut row = row.split_whitespace();
        let laddr_lport = row.nth(2)?;
        let raddr_rport = row.next()?;
        let process = row.next()?;

        let mut laddr_lport = laddr_lport.split(':');
        let laddr = laddr_lport.next()?;
        let lport = laddr_lport.next()?;

        let mut raddr_rport = raddr_rport.split(':');
        let raddr = raddr_rport.next()?;
        let rport = raddr_rport.next()?;

        let process = process.split('\"').nth(1)?.split('\"').next()?;
        let net_entry: &mut Vec<Connection> = net_table
            .entry(process.to_string())
            .or_insert_with(Vec::new);
        net_entry.push(Connection::new(laddr, lport, raddr, rport));

        Some(())
    };

    for row in raw_net_table.lines().skip(1) {
        let _ = parse(row);
    }

    Ok(net_table)
}

#[derive(Debug)]
pub struct Connection {
    laddr: String,
    pub lport: String,
    raddr: String,
    rport: String,
}

impl Connection {
    fn new(laddr: &str, lport: &str, raddr: &str, rport: &str) -> Connection {
        Connection {
            laddr: laddr.to_string(),
            lport: lport.to_string(),
            raddr: raddr.to_string(),
            rport: rport.to_string(),
        }
    }
}
