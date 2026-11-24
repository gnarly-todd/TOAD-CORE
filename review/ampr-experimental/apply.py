#!/usr/bin/env python3
"""Patch only the reviewed upstream AMPR-EMU-NG v0.5.1 source tree.

This builds a separate, clearly experimental SPRX. The official v0.5.1
vendor binary in TITAN CORE is never modified by this script.
"""
from pathlib import Path
import subprocess
import sys

UPSTREAM_COMMIT = "e1789c11bf36c2f4e9b67c5452935c1816674a2c"


def replace_once(path: Path, before: str, after: str) -> None:
    source = path.read_text()
    if source.count(before) != 1:
        raise SystemExit(f"Unsupported AMPR revision or already patched: {path}")
    path.write_text(source.replace(before, after))


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: apply.py <ampr-emu-ng-v0.5.1-checkout>")
    root = Path(sys.argv[1]).resolve()
    sha = subprocess.check_output(
        ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
    ).strip()
    if sha != UPSTREAM_COMMIT:
        raise SystemExit(f"Refusing unknown AMPR upstream revision {sha}")

    schedule = root / "crates/ampr-sys/src/rt/pack/schedule.rs"
    replace_once(
        schedule,
        """    let reserved = worker_is_latency_reserved_locked(state, worker_index);
    for _ in 0..7 {
        let lane = SCHEDULE[((*state).queue_pick_counter % 7) as usize] as u32;
        (*state).queue_pick_counter = (*state).queue_pick_counter.wrapping_add(1);
        for work_class in CLASS_ORDER {""",
        """    let reserved = worker_is_latency_reserved_locked(state, worker_index);
    // This worker holds PACK_M: no producer can add a job while we inspect
    // these lanes. Rechecking a lane 4 times on an empty/high-idle pass is
    // wasted work. Preserve the exact 4:2:1 cursor and FIFO behavior.
    let mut checked_lanes = 0u8;
    for _ in 0..7 {
        let lane = SCHEDULE[((*state).queue_pick_counter % 7) as usize] as u32;
        (*state).queue_pick_counter = (*state).queue_pick_counter.wrapping_add(1);
        let bit = 1u8 << lane;
        if checked_lanes & bit != 0 {
            continue;
        }
        checked_lanes |= bit;
        for work_class in CLASS_ORDER {""",
    )
    replace_once(
        schedule,
        """    for lane in (0..3u32).rev() {
        for work_class in CLASS_ORDER {
            if reserved && work_class == PackWorkClass::Bulk {
                continue;
            }
            let index = queue_pop_lane_locked(state, lane, work_class, retained);
            if index != INVALID_INDEX {
                return index;
            }
        }
    }
    INVALID_INDEX
}

""",
        """    // The seven weighted slots already include all three lanes. A second
    // unweighted 3-lane sweep cannot find a job and wastes mutex hold time.
    INVALID_INDEX
}

""",
    )
    replace_once(
        schedule,
        """    #[test]
    fn retained_jobs_pop_first_in_fifo_order() {""",
        """    #[test]
    fn empty_schedule_preserves_weighted_cursor() {
        let state = queue_state();
        unsafe {
            assert_eq!(queue_pop_scheduled_locked(state, 0, false), INVALID_INDEX);
            assert_eq!((*state).queue_pick_counter, 7);
            assert_eq!(queue_pop_scheduled_locked(state, 0, true), INVALID_INDEX);
            assert_eq!((*state).queue_pick_counter, 14);
            std::alloc::dealloc(state.cast(), std::alloc::Layout::new::<PackState>());
        }
    }

    #[test]
    fn retained_jobs_pop_first_in_fifo_order() {""",
    )

    tune = root / "crates/ampr-sys/src/rt/tune.rs"
    replace_once(
        tune,
        "pub(crate) const CACHE_PROTECT_HITS_DEFAULT: u32 = 3;",
        """// TITAN experimental: protect repeatedly used chunks one hit earlier.
// This is a deliberately reversible A/B setting, NOT a measured improvement.
pub(crate) const CACHE_PROTECT_HITS_DEFAULT: u32 = 2;""",
    )
    replace_once(
        tune,
        "pub(crate) const CACHE_PROTECT_PCT_DEFAULT: u32 = 50;",
        """// TITAN experimental: reserve up to 60% (upstream: 50%) for hot chunks.
// Cold sequential and random game workloads still need hardware comparison.
pub(crate) const CACHE_PROTECT_PCT_DEFAULT: u32 = 60;""",
    )
    print(f"Applied TITAN AMPR experimental scheduling/cache patch to {sha}")


if __name__ == "__main__":
    main()
