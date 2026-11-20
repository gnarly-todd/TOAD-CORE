# Firmware 13.60 A53 research track (read-only)

This is **not an A53 exploit**, an A53 patcher, or native FPKG support. It is a fail-closed offline validator for the 13.60 SDBGP read-result format and known stock instruction sites. It never executes console code or writes A53 memory. TOAD-CORE's PKG route remains AMPR; no native A53 feature flag is available.

## Evidence base

- [13.60 Pro SDBGP research](https://github.com/gangg111/ps5-a53-1360-research), particularly `docs/HANDOVER-A53-13.60-EN.md`: exact read header, transferred-byte and status checks, measured window, and stock words.
- [kstuff-lite issue #97](https://github.com/EchoStretch/kstuff-lite/issues/97): host-to-A53 memory-write command is absent on measured 13.60.
- [cragson A53 PoC](https://github.com/cragson/a53-code-exec): older read/write PoC is patched after 11.40.

The public measurements were made on PS5 Pro. Do not apply model-specific offsets elsewhere without independent on-console evidence.

## Host-only tests

```sh
python3 -m unittest discover -s review/a53-1360 -p 'test_*.py' -v
```

To validate a previously acquired SDBGP result capture (never commit firmware dumps or personal logs):

```sh
python3 review/a53-1360/verify_readback.py capture.bin --firmware 13.60 --offset 0x100 --address 0x57c14 --size 4 --stock-site 0x57c14
```

The example offset is arbitrary. `--offset` must be the **actual offset of a 0x28-byte result header** in that capture; it is not the A53 physical address. Use `--expected-sha256` only with independently trusted reference bytes. A consistent read header can be fabricated and does not attest console identity, code execution, or native package handling.

## Required gates, all currently incomplete

1. Obtain repeatable, on-console read-only evidence of A53 firmware and model identity; verify returned status, transferred-byte count, address echo, and code-site bytes.
2. Discover an independent, reversible A53 code-execution primitive on firmware 13.60 with strict firmware identification, precondition verification, post-action readback, and full restoration. No such primitive is bundled here.
3. Test the native processor path using authorized disposable homebrew packages with clean rollback and saves/DLC preservation. Demonstrate actual native processing, not AMPR software emulation.
4. Only then add an opt-in native router that cannot interfere with running games, ShadowMount ownership or existing AMPR-based packages.

Never install speculative patches or treat a successful read as proof of execution. No runtime package routing is modified by these research files.
