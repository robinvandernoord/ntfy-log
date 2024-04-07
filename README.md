# ntfy-log

Very simple CLI tool to log the result of a shell command to ntfy.sh

## Installation

### Prebuilt binaries

[Intel/AMD 64 bit (x64_64)](https://download.s3.su6.nl/x86_64/ntfy-log)  
[ARM (aarch64)](https://download.s3.su6.nl/aarch64/ntfy-log)

### Build from source

```bash
git clone https://github.com/robinvandernoord/ntfy-log.git
cd ntfy-log
cargo build --release
```

## Usage

```bash
# signature
ntfy-log <topic> [--endpoint <endpoint>] [--title <title>] <subcommand>...

# example 1 - simple
ntfy-log some-channel ls

# example 2 - advanced
ntfy-log --endpoint ntfy.s3.su6.nl --title "Custom Title" secret-channel ls -alh
```

`--endpoint`: by default this will point to `ntfy.sh`
`--title`: by dfeault this will simply be the command (e.g. `ls` in example 1)

After executing `subcommand`, a JSON result will be sent to the provided topic, with the `command,` `stdout`, `stderr`,
and `exit_code`.
If the exit code is non-zero (indicating an error), the priority will be `High`.
In addition, a second message containing simply the `title` is posted to `$topic--success` or `$topic--failure`.

The original stdout and stderr are still printed (unless you pass `--quiet/-q`) and the exit code is forwarded. You can control the output level of `ntfy-log` logs by setting the verbosity level (default: errors only; `-v`: warnings too; `-vv`: informative messages too; `-vvv`: debug messages too; `-vvvv`: stack-trace level logging).

### self-update
You can use the `ntfy-log --self-update` subcommand to download the latest binary (if a newer version is available). This binary will be downloaded from [https://download.s3.su6.nl/].
One can see the currently installed version with `ntfy-log --version`.

## Roadmap

- Complex commands containing pipes and other operators are currently not supported. It would be nice to add those. For now, you can pipe those into ntfy-log as follows: `my-command | with-pipes | ntfy-log <topic>`. Note: only stdout is recorded when using this method, and the exit code will always be 0 as this can not be determined from stdin.

