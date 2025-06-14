# rssto

![Build](https://github.com/YGGverse/rssto/actions/workflows/build.yml/badge.svg)
[![Dependencies](https://deps.rs/repo/github/YGGverse/rssto/status.svg)](https://deps.rs/repo/github/YGGverse/rssto)
[![crates.io](https://img.shields.io/crates/v/rssto.svg)](https://crates.io/crates/rssto)

## Aggregate RSS feeds into different formats

A simple multi-source feed aggregator that outputs static files in multiple formats.

## Roadmap

* [x] HTML
* [ ] Markdown
* [ ] Gemtext

## Install

``` bash
cargo install rssto
```

## Launch

``` bash
rssto --source https://path/to/source1.rss\
      --target /path/to/source1dir\
      --source https://path/to/source2.rss\
      --target /path/to/source2dir\
      --format html
```

### Options

``` bash
-d, --debug <DEBUG>              Show output (`d` - debug, `e` - error, `i` - info) [default: ei]
-f, --format <FORMAT>            Export formats (`html`,`md`,etc.) [default: html]
-l, --limit <LIMIT>              Limit channel items (unlimited by default)
-s, --source <SOURCE>            RSS feed URL(s)
    --target <TARGET>            Destination directory
    --template <TEMPLATE>        Path to template directory [default: template]
    --time-format <TIME_FORMAT>  Use custom time format [default: "%Y/%m/%d %H:%M:%S %z"]
-u, --update <UPDATE>            Update timeout in seconds [default: 60]
-h, --help                       Print help
-V, --version                    Print version
```

### Autostart

#### systemd

1. Install `rssto` by copy the binary compiled into the native system apps destination:

  * Linux: `sudo cp /home/user/.cargo/bin/rssto /usr/local/bin`

2. Create `systemd` configuration file:

``` rssto.service
# /etc/systemd/system/rssto.service

[Unit]
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=rssto
Group=rssto
ExecStart=/usr/local/bin/rssto  --source https://path/to/source1.rss\
                                --target /path/to/source1dir\
                                --source https://path/to/source2.rss\
                                --target /path/to/source2dir\
                                --format html
                                --time-format %%Y/%%m/%%d %%H:%%M:%%S

[Install]
WantedBy=multi-user.target
```
* on format time, make sure `%` is escaped to `%%`

3. Run in priority:

  * `systemctl daemon-reload` - reload systemd configuration
  * `systemctl enable rssto` - enable new service
  * `systemctl start rssto` - start the process
  * `systemctl status rssto` - check process launched
