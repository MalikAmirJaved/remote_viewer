#!/usr/bin/env bash
# mobile-3g: 300 ms RTT, 5% loss, 400 kbit
set -euo pipefail
IFACE="${IFACE:-eth0}"
sudo tc qdisc del dev "$IFACE" root 2>/dev/null || true
sudo tc qdisc add dev "$IFACE" root handle 1: htb default 10
sudo tc class add dev "$IFACE" parent 1: classid 1:10 htb rate 400kbit ceil 400kbit
sudo tc qdisc add dev "$IFACE" parent 1:10 handle 10: netem delay 150ms loss 5%
echo "mobile-3g applied on $IFACE"
