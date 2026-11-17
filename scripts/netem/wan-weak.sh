#!/usr/bin/env bash
# wan-weak: 150 ms RTT, 3% loss, 1 mbit
set -euo pipefail
IFACE="${IFACE:-eth0}"
sudo tc qdisc del dev "$IFACE" root 2>/dev/null || true
sudo tc qdisc add dev "$IFACE" root handle 1: htb default 10
sudo tc class add dev "$IFACE" parent 1: classid 1:10 htb rate 1mbit ceil 1mbit
sudo tc qdisc add dev "$IFACE" parent 1:10 handle 10: netem delay 75ms loss 3%
echo "wan-weak applied on $IFACE"
