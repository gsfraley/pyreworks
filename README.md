# Pyreworks - Logi G560 Speaker LED lights

This is a CLI controller for the LED colors on Logitech G560 Speaker Systems, written in Rust and leveraging Tokio and nusb.

**Strong credit goes to https://github.com/mijoe/g560-led for documenting the control codes and showing sensible use.**


## Installation

If you have Cargo, that's the quickest way to get the binary:

```sh
cargo install pyreworks-ctl
```

You can also grab binaries from [the latest release](https://github.com/gsfraley/pyreworks/releases/latest).

## Usage

Simply put the `pyrectl` binary on the `$PATH` and run it with sudo permissions!  Currently there are direct setting controls for the G560's display LEDs:

```
# pyreworks g560 set --help

Set mode and/or state for speaker lights

Usage: pyrectl g560 set [COMMAND]

Commands:
  off      Set speaker lights off
  solid    Set speaker lights to solid color
  breathe  Set speaker lights to a breathing color
  cycle    Set speaker lights to cycling colors
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
```sh
pyrectl g560 set off                              # Basic command
pyrectl g560 set off -t left-front -t right-back  # Target specific speakers
pyrectl g560 set solid -c 'turquoise'             # Set colors by CSS value, e.g. `'#FFFFFF'`, or `'rgb(128, 255, 0)'`, or 'navy'
pyrectl g560 set breathe -c '#B5711F'             # Set breathing colors
pyrectl g560 set cycle -r 30s                     # Rotate through a rainbow of colors every 30 seconds
```

## Ambitions
I left this semi-generically named ("pyreworks") in case I wanted to revisit and expand with support for new devices or new features.  I don't have much in the way of devices to test, but there were two things I was thinking about trying with this:
* Remake it as a systemd service and control client to better support things like suspend behavior and presets.
* Hook in a Pipewire stream of the desktop to actually make it match the desktop colors in XDG systems (hopefully make it Flatpak compatible).
