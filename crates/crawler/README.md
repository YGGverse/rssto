# rssto-crawler

## Install

``` bash
git clone https://github.com/YGGverse/rssto.git
cd rssto
cargo build --release
```

## Launch

``` bash
rssto-crawler -c config/example.toml
```
> [!TIP]
> * prepend `RUST_LOG=rssto_crawler=trace` to print worker details (supported [levels](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.LevelFilter.html))
>     * or just `RUST_LOG=trace` to debug all components in use
> * append `-u TIME` to run as the daemon with `TIME` interval update
> * see `rssto-crawler --help` to print all available options

### Systemd

1. Install `rssto-crawler` by copy the binary compiled into the native system apps destination:
    * Linux: `sudo install target/release/rssto-crawler /usr/local/bin/rssto-crawler`
2. Create `systemd` configuration file at `/etc/systemd/system/rssto-crawler.service`:

``` rssto-crawler.service
[Unit]
After=network-online.target
Wants=network-online.target

[Service]
Type=simple

User=rssto
Group=rssto

# Uncomment for debug
# Environment="RUST_LOG=rssto_crawler=debug"
# Environment="NO_COLOR=1"

ExecStart=/usr/local/bin/rssto-crawler -c /path/to/config.toml

StandardOutput=file:///home/rssto/crawler-debug.log
StandardError=file:///home/rssto/crawler-error.log

[Install]
WantedBy=multi-user.target
```
* example above requires new system user (`useradd -m rssto`)

3. Run in priority:

* `systemctl daemon-reload` - reload systemd configuration
* `systemctl enable rssto-crawler` - enable new service
* `systemctl start rssto-crawler` - start the process
* `systemctl status rssto-crawler` - check process launched