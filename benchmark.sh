#!/usr/bin/env bash
set -euo pipefail

# Benchmark: rust-chat Iced vs Makepad vs egui
# Measures: binary size, startup time, idle CPU%, RSS memory, GPU% (via powermetrics)

DURATION=12       # seconds to sample
WARMUP=3          # seconds before measuring
DIR="$(cd "$(dirname "$0")" && pwd)"
ICED="$DIR/target/release/rust-chat-iced"
MAKEPAD="$DIR/target/release/rust-chat-makepad"
EGUI="$DIR/target/release/rust-chat-egui"

divider() { printf '\n%s\n' "$(printf '=%.0s' {1..60})"; }

binary_size() {
    local bin=$1 name=$2
    local bytes
    bytes=$(stat -f%z "$bin" 2>/dev/null || stat -c%s "$bin" 2>/dev/null)
    local mb
    mb=$(echo "scale=2; $bytes / 1048576" | bc)
    printf '  %-12s %s MB  (%s bytes)\n' "$name" "$mb" "$bytes"
}

measure_app() {
    local bin=$1 name=$2
    echo ""
    echo "--- $name ---"

    # Startup time
    local t0 t1 startup_ms
    t0=$(python3 -c 'import time; print(time.time())')
    "$bin" &>/dev/null &
    local pid=$!

    # Wait for window (process exists and has reasonable RSS)
    for i in $(seq 1 40); do
        if ps -p $pid -o rss= &>/dev/null; then
            local rss_now
            rss_now=$(ps -p $pid -o rss= 2>/dev/null | tr -d ' ')
            if [[ -n "$rss_now" ]] && (( rss_now > 5000 )); then
                break
            fi
        fi
        sleep 0.1
    done
    t1=$(python3 -c 'import time; print(time.time())')
    startup_ms=$(python3 -c "print(int(($t1 - $t0) * 1000))")
    printf '  Startup:     ~%s ms\n' "$startup_ms"

    # Warmup
    sleep $WARMUP

    # Sample CPU + Memory over DURATION
    local cpu_sum=0 mem_sum=0 samples=0
    for i in $(seq 1 $DURATION); do
        local line
        line=$(ps -p $pid -o %cpu=,rss= 2>/dev/null || true)
        if [[ -n "$line" ]]; then
            local cpu rss
            cpu=$(echo "$line" | awk '{print $1}')
            rss=$(echo "$line" | awk '{print $2}')
            cpu_sum=$(echo "$cpu_sum + $cpu" | bc)
            mem_sum=$(echo "$mem_sum + $rss" | bc)
            samples=$((samples + 1))
        fi
        sleep 1
    done

    if (( samples > 0 )); then
        local avg_cpu avg_mem_mb
        avg_cpu=$(echo "scale=1; $cpu_sum / $samples" | bc)
        avg_mem_mb=$(echo "scale=1; $mem_sum / $samples / 1024" | bc)
        printf '  CPU (idle):  %s%%\n' "$avg_cpu"
        printf '  RAM (idle):  %s MB\n' "$avg_mem_mb"
    else
        echo "  (process exited before measurement)"
    fi

    # GPU sampling via powermetrics (needs sudo, skip if unavailable)
    if sudo -n true 2>/dev/null; then
        local gpu_out
        gpu_out=$(sudo powermetrics --samplers gpu_power -i 2000 -n 1 2>/dev/null || true)
        if [[ -n "$gpu_out" ]]; then
            local gpu_active
            gpu_active=$(echo "$gpu_out" | grep -i "active residency" | head -1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 || echo "n/a")
            printf '  GPU active:  %s\n' "$gpu_active"
        fi
    else
        printf '  GPU active:  (needs sudo for powermetrics)\n'
    fi

    # Thread count
    local threads
    threads=$(ps -M -p $pid 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
    printf '  Threads:     %s\n' "$threads"

    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
}

echo "========================================"
echo " rust-chat Framework Benchmark"
echo " Machine: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || uname -m)"
echo " GPU: $(system_profiler SPDisplaysDataType 2>/dev/null | grep 'Chipset Model' | awk -F: '{print $2}' | xargs)"
echo " Date: $(date '+%Y-%m-%d %H:%M')"
echo "========================================"

divider
echo "BINARY SIZE"
binary_size "$ICED" "Iced"
binary_size "$MAKEPAD" "Makepad"
binary_size "$EGUI" "egui"

divider
echo "RUNTIME METRICS (idle, ${DURATION}s sample after ${WARMUP}s warmup)"

measure_app "$ICED" "Iced"
measure_app "$MAKEPAD" "Makepad"
measure_app "$EGUI" "egui"

divider
echo "DONE"
