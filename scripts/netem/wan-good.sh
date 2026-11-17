#!/usr/bin/env bash
# wan-good: 40 ms RTT (20 ms each way), 0.1% loss, 20 mbit ceiling
set -euo pipefail
IFACE="${IFACE:-eth0}"
sudo tc qdisc del dev "$IFACE" root 2>/dev/null || true
sudo tc qdisc add dev "$IFACE" root handle 1: htb default 10
sudo tc class add dev "$IFACE" parent 1: classid 1:10 htb rate 20mbit ceil 20mbit
sudo tc qdisc add dev "$IFACE" parent 1:10 handle 10: netem delay 20ms loss 0.1%
echo "wan-good applied on $IFACE"
