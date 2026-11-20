# Experimental A53 readback ELF build: PS5 Pro, firmware 13.60

> **STOP for original PS5 and PS5 Slim:** This experiment is not validated on your hardware. The owner of TOAD-CORE uses an original PS5. Do not build or load this Pro payload. The builder now refuses execution by default. There is presently no verified original-PS5 A53 execution/readback ELF in this repository.


This is a **separate research ELF builder**, not A53 code execution, PPR patching, or native FPKG support. It makes no changes to TOAD-CORE's loader. The build recipe has **not been compiled in this chat environment or validated on hardware**.

Source: https://github.com/gangg111/ps5-a53-1360-research (pinned commit 13aa9d900ab87eb4ec89191091131f9b7620d243, `tools/a53layout/a53layout.c`). The builder fetches the original source at build time, rather than importing code without a license into this fork, and adds a firmware-13.60 gate.

## On iPhone, in an existing TOAD-CORE Codespace terminal

Paste this as one command:

```sh
git pull --ff-only && TOAD_CONFIRMED_PS5_PRO_1360=YES bash review/a53-1360/build-console-probe.sh
```

Only on independently confirmed **PS5 Pro hardware** should the explicit `TOAD_CONFIRMED_PS5_PRO_1360=YES` acknowledgement be used. This is a build acknowledgement, **not** a runtime hardware model check. The command compiles using the existing PS5 SDK Docker image (building that image if absent) and creates `TOAD-A53-READBACK-1360-PRO.elf` at the repository root. In VS Code Explorer, use the file's context menu > Download to save it to iPhone Files. Do not confuse this file with `target/ps5/titan-core.elf`.

## Dangerous operation; model warning

The public research was performed **only on PS5 Pro firmware 13.60**. Original/Slim hardware has not been validated. **Do not load this ELF on original/Slim models.** It uses kernel-memory and MP4 mailbox register writes to request A53 *reads*, temporarily changes process authorization, and could hang or crash the console. The builder checks the firmware at runtime but cannot verify the hardware model. This is not a harmless passive diagnostics tool.

Use a clean reboot, compatible jailbreak, kstuff and an ELF loader, with games closed. Do not start TOAD-CORE's combined stack or ShadowMountPlus at the same time, and do not autoload the experiment. A console-side Payload Manager (http://PS5_IP:8084) can accept a file upload from Safari on the same LAN and launch it. Review your console's known-good recovery procedures first.

Upstream log paths are `/data/a53layout.txt`, `/data/a53_layout.bin`, and `/data/a53_dev.bin`. The binary outputs may contain proprietary firmware; do not share them publicly. You can retrieve the text log using a separately launched PS5 file manager (e.g. http://PS5_IP:8888) after the experiment. Do not claim success merely because a console process exited. Verify each result's address echo, status, and transferred bytes.

Neither this tool nor the host-only unit tests establishes 13.60 A53 code execution or changes game package routing. Current TOAD-CORE continues using AMPR.
