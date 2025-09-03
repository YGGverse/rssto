# rssto

![Build](https://github.com/YGGverse/rssto/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/rssto/status.svg)](https://deps.rs/repo/github/YGGverse/rssto)
[![crates.io](https://img.shields.io/crates/v/rssto.svg)](https://crates.io/crates/rssto)

Convert RSS feeds into multiple formats

## Features

* [x] Multiple feed sources with flexible TOML config options
    * [x] Limit channel items
    * [x] Format time
    * [x] Multiple export format definition
* [x] Custom templates
* [x] Single export or daemon mode with update time
* Export formats:
    * [x] HTML
    * [x] [Gemtext](https://geminiprotocol.net/docs/gemtext.gmi)
    * [ ] JSON
    * [ ] Markdown

## Install

``` bash
cargo install rssto
```

## Launch

``` bash
rssto -c config/example.toml
```
> [!TIP]
> * prepend `RUST_LOG=DEBUG` to print worker details (supported [levels](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.LevelFilter.html))
> * append `-u TIME` to run as the daemon with `TIME` interval update
> * see `rssto --help` to print all available options

### Autostart

#### systemd

1. Install `rssto` by copy the binary compiled into the native system apps destination:

  * Linux: `sudo install /home/user/.cargo/bin/rssto /usr/local/bin/rssto`

2. Create `systemd` configuration file at `/etc/systemd/system/rssto.service`:

``` rssto.service
[Unit]
After=network-online.target
Wants=network-online.target

[Service]
Type=simple

User=rssto
Group=rssto

# Uncomment for debug
# Environment="RUST_LOG=DEBUG"
# Environment="NO_COLOR=1"

ExecStart=/usr/local/bin/rssto -c /path/to/config.toml

StandardOutput=file:///home/rssto/debug.log
StandardError=file:///home/rssto/error.log

[Install]
WantedBy=multi-user.target
```
* example above requires new system user (`useradd -m rssto`)

3. Run in priority:

  * `systemctl daemon-reload` - reload systemd configuration
  * `systemctl enable rssto` - enable new service
  * `systemctl start rssto` - start the process
  * `systemctl status rssto` - check process launched
