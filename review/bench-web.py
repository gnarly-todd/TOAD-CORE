#!/usr/bin/env python3
"""Measure TITAN CORE web UI/API response times from a PC on the PS5 LAN.

This does not measure native game-loading time or FPS. Read-only HTTP GETs only.
"""
import argparse
import json
import statistics
import time
import urllib.error
import urllib.request


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("url", help="e.g. http://192.168.1.100:8097")
    parser.add_argument("--samples", type=int, default=7)
    args = parser.parse_args()
    base = args.url.rstrip("/")
    if not base.startswith(("http://", "https://")) or not (1 <= args.samples <= 100):
        parser.error("Provide an http(s) URL and 1-100 samples")
    for endpoint in ("/", "/api/state", "/titan.appcache"):
        elapsed = []
        payloads = []
        for i in range(args.samples):
            start = time.perf_counter()
            try:
                with urllib.request.urlopen(
                    urllib.request.Request(base + endpoint, headers={"Cache-Control": "no-cache"}),
                    timeout=15,
                ) as response:
                    data = response.read()
                    if response.status != 200:
                        raise RuntimeError(f"HTTP {response.status}")
            except (OSError, RuntimeError) as e:
                raise SystemExit(f"{endpoint} request {i + 1} failed: {e}") from e
            elapsed.append((time.perf_counter() - start) * 1000)
            payloads.append(len(data))
        sorted_ms = sorted(elapsed)
        p95 = sorted_ms[max(0, (len(sorted_ms) * 95 + 99) // 100 - 1)]
        print(json.dumps({
            "endpoint": endpoint,
            "samples": len(elapsed),
            "median_ms": round(statistics.median(elapsed), 2),
            "p95_ms": round(p95, 2),
            "min_ms": round(sorted_ms[0], 2),
            "max_ms": round(sorted_ms[-1], 2),
            "bytes": min(payloads),
            "max_bytes": max(payloads),
        }))
    print("Network + HTTP measurements only; launch time, game FPS and disk throughput are not measured.")


if __name__ == "__main__":
    main()
