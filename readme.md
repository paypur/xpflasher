# XpFlasher

XpFlasher is a command-line tool for flashing Sony Xperia devices using firmware files downloaded with XperiFirm.

It is a Rust rewrite of newflasher, with a focus on correctness, reliability, and improved flashing speed.

Because XpFlasher is based on newflasher, it is expected to support Xperia XZ1 Premium and newer devices.
However, not all devices have been tested yet. Check the list of for your [device](#validated-devices)

___

## Installation and Setup

### Arch Linux

XpFlasher is available in the AUR.

```bash
yay -S xpflasher
```

### Other Linux distros

Other package formats may come later.

1. Download the latest release from the [Releases](https://github.com/paypur/xpflasher/releases) tab
2. Copy the binary to which ever directory your distro uses for user installed binaries

### Windows

1. Download the latest release from the [Releases](https://github.com/paypur/xpflasher/releases) tab
2. Download [Zadig](https://zadig.akeo.ie/)
3. Shutdown your device connect your device to your computer using a USB cable while holding the volume down button to enter flashmode (a green *)
4. Using Zadig, install WinUSB as the driver for your device

## Usage

___

> [!WARNING]
> Backup your data before attempting to flash your device. In case flashing fails, your device may become corrupted (some cases are still recoverable). You don't
want to learn the hard way

1. Download the firmware for your specific device and region using XperiFirm

> [!WARNING]
> Make sure you download the correct firmware for your device and region. If you download the wrong firmware, your device may become corrupted and you may need to
perform a factory reset to recover it.

2. Shutdown your device and connect your device to your computer using a USB cable (preferably 3.0) while holding the volume down button to enter flashmode (a
   green *)
3. In a terminal run

```bash
xpf [path/to/firmware/files]
```

4. XpFlasher will prompt you to continue when it is prepared to flash your device.

## Validated devices

___

- Xperia 1 VI
- Xperia XZ2 Compact

This list is very short for now since these are the only devices I have.

If you have a device that is not listed that you are willing to test for me,
please create an issue with a log file of a successful flash.
