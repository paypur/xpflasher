use anyhow::anyhow;
use log::{debug, error, log_enabled, trace, Level, info};
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use nusb::Speed;
use tar::Entry;

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

pub fn noerase_in_updatexml(file_name: &OsStr) -> bool {
    let search_for = file_name.to_str().unwrap();

    let file = match File::open("update.xml") {
        Ok(f) => f,
        Err(e) => {
            error!("{}", e);
            return false;
        },
    };

    let reader = BufReader::new(file);

    for line in reader.lines().into_iter() {
        match line {
            Ok(str) => {
                if !str.is_empty() && str.trim() == format!("<NOERASE>{search_for}</NOERASE>") {
                    return true;
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }

    false
}

pub fn tar_entry_header_name<'a>(entry: &'a Entry<Box<dyn Read>>) -> anyhow::Result<&'a str> {
    let header = entry.header().as_ustar().ok_or_else(|| anyhow!("Failed to parse Tar Entry header as ustar"))?;
    Ok(str::from_utf8(&header.name)?)
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