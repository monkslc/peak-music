set -euo pipefail

(cd backend && cargo test)
(cd backend && cargo build --release --target x86_64-unknown-linux-musl)

scp backend/systemd/peak-music.service ec2-user@api.peak.band:
ssh ec2-user@api.peak.band "sudo mv peak-music.service /etc/systemd/system"

scp backend/target/x86_64-unknown-linux-musl/release/peak-music ec2-user@api.peak.band:
ssh ec2-user@api.peak.band "sudo mv peak-music /usr/bin"
