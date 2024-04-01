# cargo build --release --target x86_64-unknown-linux-gnu
# cargo build --release --target aarch64-unknown-linux-gnu

# see `rclone config` -> garage-s3-ntfy-log

rclone copy --progress target/x86_64-unknown-linux-gnu/release/ntfy-log garage-s3-ntfy-log:ntfy-log/x86_64/
rclone copy --progress target/aarch64-unknown-linux-gnu/release/ntfy-log garage-s3-ntfy-log:ntfy-log/aarch64/

# e.g. https://download.s3.su6.nl/aarch64/ntfy-log
