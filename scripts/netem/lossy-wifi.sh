#!/usr/bin/env bash
# lossy-wifi: 30 ms base + 40 ms jitter, 8% loss, 5 mbit
set -euo pipefail
IFACE="${IFACE:-eth0}"
sudo tc qdisc del dev "$IFACE" root 2>/dev/null || true
sudo tc qdisc add dev "$IFACE" root handle 1: htb default 10
sudo tc class add dev "$IFACE" parent 1: classid 1:10 htb rate 5mbit ceil 5mbit
sudo tc qdisc add dev "$IFACE" parent 1:10 handle 10: netem delay 30ms 40ms loss 8%
echo "lossy-wifi applied on $IFACE"
