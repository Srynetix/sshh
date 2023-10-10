# sshh: Simple SSH TUI using your SSH configuration file

## Usage

Just setup some hosts in your `~/.ssh/config` file, then run

    sshh

Select the host you want to connect to, press ENTER and _tada_, you are
connected to the host.

![animation](./doc/demo.gif)
> Made with [vhs](https://github.com/charmbracelet/vhs) using this [tape](./doc/demo.tape).

## Installation

For now you will need [Rust](https://rust-lang.org).

Then run:

    cargo install --git https://github.com/Srynetix/sshh