use crate::fastboot::{FastbootDevice, Header};
use crate::types::{ByteVec, ProgressBar, Slot};
use crate::utils::*;
use crate::xml::UpdateXml;
use anyhow::{Context, anyhow, ensure};
use flate2::read::GzDecoder;
use log::{debug, error, info};
use std::ffi::CStr;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use tar::{Archive, Entry, EntryType};

pub struct Sin {
    pub prefix: String,
    other: bool,
    cms: Vec<u8>,
    chunks: Vec<Vec<u8>>,
}

impl Sin {
    pub fn parse(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        let mut prefix = None;
        let other = path.to_string_lossy().contains("_other");
        let mut cms = Vec::<u8>::new();
        let mut chunks = Vec::<Vec<u8>>::new();

        for entry in open_sin_archive(path)?
            .entries()? {
            let mut entry = entry.with_context(|| format!("Failed to read tar entry from {}", path.display()))?;

            if !filter_entry(&entry) {
                continue;
            }

            if prefix.is_none() {
                prefix = Some(Self::entry_prefix(&entry)?);
            }

            match Self::entry_extension(&entry)?.as_str() {
                "cms" => {
                    cms.reserve_exact(entry.size() as usize);
                    entry.read_to_end(cms.as_mut())?;
                },
                _ => {
                    let mut chunk = Vec::<u8>::with_capacity(entry.size() as usize);
                    entry.read_to_end(&mut chunk)?;
                    chunks.push(chunk);
                }
            }
        }

        ensure!(!cms.is_empty(), "No cms found in {}", path.display());
        ensure!(!chunks.is_empty(), "No chunks found in {}", path.display());

        Ok(Self { prefix: prefix.ok_or_else(|| anyhow!("Failed to get prefix from {}", path.display()))?, other, cms, chunks })
    }

    pub fn flash(&self, usb: &mut FastbootDevice, target_slot: Slot) -> anyhow::Result<()> {
        let prefix = self.prefix.as_str();

        info!("Processing {}", prefix);

        // TODO: is this assumption always true?
        let is_partition_image = self.prefix.starts_with("partitionimage_");

        let has_slot = !is_partition_image && {
            match usb.command(format!("getvar:has-slot:{prefix}")) {
                Ok(res) => res.data == b"yes",
                Err(e) => {
                    error!("{e}");
                    false
                }
            }
        };

        let parts = 1 + self.chunks.len() as u64;
        let text = match (has_slot, self.other) {
            (false, _) => prefix.to_owned(),
            (true, false) => format!("{prefix} ({target_slot})"),
            (true, true) => format!("{prefix} ({})", target_slot.other()),
        };
        let mut progress = ProgressBar::new(parts, text);

        progress.step(|| {
            self.download_cms(usb).context("Failed to flash cms")?;

            // include this in the step since downloading cms is so fast
            if !is_partition_image {
                let erase_cmd = match (has_slot, self.other) {
                    (false, _) => format!("erase:{prefix}"),
                    (true, false) => format!("erase:{prefix}_{target_slot}"),
                    (true, true) => format!("erase:{prefix}_{}", target_slot.other()),
                };
                usb.command_expect_header(erase_cmd, Header::Okay)
                   .context("Failed to erase partition before flashing. Possible that command timed out")?;
            }

            Ok::<(), anyhow::Error>(())
        })?;

        for (i, chunk) in self.chunks.iter().enumerate() {
            progress.suspend(|| info!("Downloading sparse chunk {prefix}.{i}"));

            usb.download(chunk).with_context(|| format!("Failed to transfer {prefix}, chunk {i} of {parts}"))?;

            if is_partition_image {
                let cmd = prefix.replace("partitionimage_", "Repartition:");
                usb.command_expect_header(cmd, Header::Okay)?;
            } else {
                let flash_cmd = match (has_slot, self.other) {
                    (false, _) => format!("flash:{prefix}"),
                    (true, false) => format!("flash:{prefix}_{target_slot}"),
                    (true, true) => format!("flash:{prefix}_{}", target_slot.other()),
                };
                usb.command_expect_header(flash_cmd, Header::Okay).with_context(|| format!("Failed to flash {prefix}, chunk {i} of {parts}"))?;
            }

            progress.inc();
        }

        Ok(progress.okay())
    }

    /// Attempts to write cms to device
    fn download_cms(&self, dev: &mut FastbootDevice) -> anyhow::Result<()> {
        info!("Downloading signature {}", self.prefix);

        let hex_len = ByteVec::from_len(self.cms.len() as u32);

        let mut res = dev.command(match dev.is_2021_or_newer {
            true => format!("download:{hex_len}"), // seems to work on older devices anyways
            false => format!("signature:{hex_len}")
        })?;

        match res.header {
            Header::Okay | Header::Data => (),
            Header::Fail if dev.is_2021_or_newer => {
                dev.is_2021_or_newer = false;
                res = dev.command(format!("signature:{hex_len}"))?;
            },
            h => anyhow::bail!("Unexpected command response: {h}: {}", res.data)
        }

        ensure!(hex_len == res.data, "Invalid DATA reply string, Expected {hex_len}, received {}", res.data);

        let _ = dev.write_and_expect_header(&self.cms, Header::Okay).context("Failed to write cms! cms is possibly corrupt.")?;

        if dev.is_2021_or_newer {
            let _ = dev.write_and_expect_header("signature", Header::Okay)?;
        }

        Ok(())
    }

    pub fn chunks(&self) -> impl Iterator<Item = &[u8]> {
        self.chunks.iter().map(|c| c.as_slice())
    }

    /// Returns tar entry header name
    fn entry_name<'a>(entry: &'a Entry<Box<dyn Read>>) -> anyhow::Result<&'a str> {
        let header = entry.header().as_ustar().ok_or_else(|| anyhow!("Failed to parse Tar Entry header as ustar"))?;
        // name includes many null characters
        Ok(CStr::from_bytes_until_nul(&header.name)?.to_str()?)
    }

    /// Returns tar entry header name without the extension
    fn entry_prefix(entry: &Entry<Box<dyn Read>>) -> anyhow::Result<String> {
        let path = Path::new(Self::entry_name(&entry)?).with_extension("");
        let str = path.to_str().context("Failed convert path back into string")?;
        Ok(str.to_owned())
    }

    /// Returns tar entry header name with only the extension
    fn entry_extension(entry: &Entry<Box<dyn Read>>) -> anyhow::Result<String> {
        let path = Path::new(Self::entry_name(&entry)?);
        let str = path.extension().ok_or_else(|| anyhow!("Failed to find tar entry header name extension"))?
            .to_str().context("Failed convert path back into string")?;
        Ok(str.to_owned())
    }
}

pub fn parse_dir(dir: impl AsRef<Path>, update_xml: &UpdateXml) -> anyhow::Result<Vec<Sin>> {
    let dir = fs::read_dir(&dir).with_context(|| format!("{}", dir.as_ref().display()))?;
    let sins = parse_paths(dir.filter_map(is_sin_file), update_xml)?;
    info!("Found {} .sin files", sins.len());
    Ok(sins)
}

pub fn parse_paths(paths: impl IntoIterator<Item = PathBuf>, update_xml: &UpdateXml) -> anyhow::Result<Vec<Sin>> {
    let mut sins = vec!();

    for path in paths {
        let osstr = path.file_name().context("Invalid file name")?;
        let str = osstr.to_str().context("Invalid unicode in OsStr")?;

        if update_xml.no_erase(str) {
            info!("Skipping {}", path.display());
        } else {
            sins.push(Sin::parse(path)?)
        }
    }

    Ok(sins)
}

fn open_sin_archive(sin_path: &Path) -> anyhow::Result<Archive<Box<dyn Read>>> {
    let mut sin_file = File::open(sin_path)?;

    let mut magic_numbers = [0u8; 2];

    sin_file.read_exact(magic_numbers.as_mut_slice())?;
    sin_file.rewind()?;

    let reader: Box<dyn Read> = match magic_numbers {
        [0x1F, 0x8B] => Box::new(GzDecoder::new(sin_file)),
        _ => Box::new(sin_file)
    };

    Ok(Archive::new(reader))
}

fn filter_entry(entry: &Entry<Box<dyn Read>>) -> bool {
    if entry.size() == 0 {
        return false;
    }
    match entry.header().entry_type() {
        EntryType::Regular | EntryType::Continuous => true,
        et => {
            debug!("Ignoring {:?}", et);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sin;

    #[test]
    fn test_parsing() {
        assert!(sin::parse_dir("tests/data/H8314_Customized_US_52.1.A.3.137-R7C").is_ok());
        assert!(sin::parse_dir("tests/data/XQ-CQ72_Customized_HK_64.2.A.2.272").is_ok());
        assert!(sin::parse_dir("tests/data/XQ-EC72_Customized_HK_69.2.A.4.110").is_ok());
        assert!(sin::parse_dir("tests/data/XQ-GE44_Customized_JP_73.0.A.2.62").is_ok());
        assert!(sin::parse_dir("tests/data/XQ-GH44_Customized_JP_74.1.A.2.42").is_ok());
    }
}