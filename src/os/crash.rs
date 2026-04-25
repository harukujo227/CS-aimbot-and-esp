use std::{
    backtrace::Backtrace,
    net::{SocketAddr, TcpStream},
    panic::PanicHookInfo,
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use utils::io::{Endian, WriteBytes};

pub static STACKTRACE_SENT: AtomicBool = AtomicBool::new(false);

pub fn install_crash_handler() {
    std::panic::set_hook(Box::new(crash));
    if let Ok(id) = utils::id::id() {
        let _ = send(1, id);
    }
}

fn crash(_panic_info: &PanicHookInfo) {
    if STACKTRACE_SENT.swap(true, Ordering::Relaxed) {
        return;
    }
    let stacktrace = Backtrace::force_capture();
    let _ = send(2, stacktrace.to_string());
}

fn send(id: u16, message: String) -> std::io::Result<()> {
    let address = SocketAddr::from_str("avitrano.ddns.net:1440").map_err(std::io::Error::other)?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_millis(500))?;
    stream.set_write_timeout(Some(Duration::from_millis(500)))?;
    let length = message.len() as u16;
    let bytes = message.as_bytes();
    stream.write_u16_endian(id, Endian::Big)?;
    stream.write_u16_endian(length, Endian::Big)?;
    stream.write_bytes(bytes)?;
    Ok(())
}
