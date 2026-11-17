#!/usr/bin/env bash
# Ideal LAN: clear any netem on $IFACE (default eth0 / ens33).
set -euo pipefail
IFACE="${IFACE:-eth0}"
sudo tc qdisc del dev "$IFACE" root 2>/dev/null || true
echo "lan: cleared netem on $IFACE"
