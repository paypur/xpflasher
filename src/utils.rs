use log::{debug, log_enabled, trace, Level};
use std::fmt::Write;
use std::fs::{DirEntry};
use std::path::PathBuf;
use nusb::Speed;

pub fn trace_formatted_hex(message: &str, buffer: &[u8]) {
    if log_enabled!(Level::Trace) {
        trace!("{}", formatted_hex(message, buffer));
    }
}

pub fn formatted_hex(message: &str, buffer: &[u8]) -> String {
    let mut builder = String::with_capacity(0xF00);

    let _ = writeln!(builder, "{}[0x{:X}]:", message, buffer.len());

    buffer.chunks(16)
          .take(64)
          .enumerate()
          .for_each(|(i, chunk)| {
              let _ = writeln!(builder, "0x{i:07X}0  {:<48} {}", chunk.iter().map(|b| format!("{b:02X} ")).collect::<String>(), u8_ascii(chunk));
          });

    builder
}

pub fn u8_ascii(line: &[u8]) -> String {
    line.iter()
        .map(|b| match *b >= 33 && *b < 127 {
            true => *b as char,
            false => '.',
        }).collect::<String>()
}

pub fn is_sin_file(entry: std::io::Result<DirEntry>) -> Option<PathBuf> {
    let path = entry.ok()?.path();
    if path.extension()? == "sin" {
        Some(path)
    } else {
        None
    }
}

pub fn is_ta_file(entry: std::io::Result<DirEntry>) -> Option<PathBuf> {
    let path = entry.ok()?.path();
    if path.extension()? == "ta" {
        Some(path)
    } else {
        None
    }
}

pub fn speed_as_data_rate(speed: Speed) -> &'static str {
    match speed {
        Speed::Low => "1.5 Mb/s",
        Speed::Full => "12 Mb/s",
        Speed::High => "480 Mb/s",
        Speed::Super => "5 Gb/s",
        Speed::SuperPlus => "10 Gb/s",
        _ => "unknown"
    }
}

/// https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ufs/ns-ufs-ufs_device_descriptor
/// https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ufs/ns-ufs-ufs_unit_descriptor
pub fn ufs_sector_count(ufs_info: &[u8]) -> anyhow::Result<u64> {
    debug!("{}", formatted_hex("UFS", ufs_info));
    let device_descriptor_length = ufs_info[0];
    let unit_descriptor = &ufs_info[(device_descriptor_length as usize)..];
    Ok(u64::from_be_bytes(unit_descriptor[0x18..=0x1F].try_into()?))
}


/// https://api.riot-os.org/structsdmmc__ext__csd__t.html
pub fn emmc_sector_count(emmc_info: &[u8]) -> anyhow::Result<u64> {
    debug!("{}", formatted_hex("EMMC", emmc_info));
    Ok(u32::from_be_bytes(emmc_info[0xd4..0xd8].try_into()?) as u64)
}