# XpFlasher

XpFlasher is a command-line tool for flashing Sony Xperia devices using firmware files downloaded with [XperiFirm](https://xdaforums.com/t/tool-xperifirm-xperia-firmware-downloader-v5-8-1.2834142/).

It is a Rust fork of newflasher, focusing on correctness, reliability, and speed.

Because XpFlasher is based on newflasher, it should support Xperia XZ1 Premium and newer devices.
However, not all devices have been tested. Check the validation list for your [device](#validated-devices)

## Installation and Setup

### Arch Linux

XpFlasher is available in the AUR.

```bash
yay -S xpflasher
```

### Other Linux distros

Other package formats may come later, but for now you will need to download the binary manually.

1. Download the latest release from the [Releases](https://github.com/paypur/xpflasher/releases) tab
2. Make the binary executable
```bash
chmod +x xpflasher
```
3. Copy the binary to *one* of the directions your distro uses for user-installed binaries
   - `mv xpflasher ~/bin/`
   - `mv xpflasher ~/.local/bin/`
   - `sudo mv xpflasher /usr/local/bin/` (requires root)

### Windows

1. Download the latest release from the [Releases](https://github.com/paypur/xpflasher/releases) tab
2. Download [Zadig](https://zadig.akeo.ie/)
3. Shutdown your device connect your device to your computer using a USB cable while holding the volume down and power button to enter flashmode (a green *)
4. Using Zadig, install WinUSB as the driver for device `Android` with USB ID: `0FCE BOOB`

## Usage

> [!WARNING]
> Backup your data before attempting to flash your device. If flashing fails, your device may become corrupted (some cases are recoverable). You don't
want to learn the hard way

1. Download the firmware for your specific device and region using [XperiFirm](https://xdaforums.com/t/tool-xperifirm-xperia-firmware-downloader-v5-8-1.2834142/)

> [!WARNING]
> Do NOT rename the parent directory or any of the files and do NOT try to flash firmware from a different region. You may brick your device

2. Shutdown your device and connect your device to your computer using a USB cable (preferably 3.0) while holding the volume down and power button to enter flashmode (a
   green *)
3. To flash firmware, run in a terminal
   - Linux
   ```bash
   xpflasher [path/to/firmware/files]
   ```
   - Windows (open terminal in the directory containing the .exe)
   ```ps1
   ./xpflasher.exe [path/to/firmware/files]
   ```
4. A prompt will ask you to continue when the firmware has been parsed and is ready to flash your device.

## Validated devices

- Xperia 1 VI
- Xperia XZ2 Compact

This list is very short for now since these are the only devices I have.

If you have a device that is not listed that you are willing to test for me,
please create an issue with a log file of a successful flash.
