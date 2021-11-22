set -euo pipefail

ssh ec2-user@api.peak.band "sudo systemctl daemon-reload"
ssh ec2-user@api.peak.band "sudo systemctl restart peak-music.service"
