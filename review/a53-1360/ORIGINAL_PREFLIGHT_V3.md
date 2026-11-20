# Original PS5 A53 preflight v3

The user's original PS5 executed v2 and reported `mp4_dump=open_readonly_failed;errno=1` (EPERM). That confirms the v2 ELF and log writer worked, but does not establish A53 access. V3 focuses on understanding the permission failure **before any A53 memory-read attempt**.

V3 reports real/effective UID and GID, supplementary group IDs, `lstat`, `stat`, `access(F_OK)` and `access(R_OK)` for `/dev`, `/dev/mp4` and `/dev/mp4/dump`, a nonblocking read-only `open` with its errno, and the same read-only KEKCALL_CHECK readiness syscall used by TITAN CORE `src/stack.rs`.

The device's Unix permissions are not proof of PS5 authorization, and successful kstuff readiness is not proof of A53 execution. This diagnostic does not change credentials, issue MP4 ioctls, read A53 memory, write kernel memory, or activate native FPKG support.

## Build on iPhone using existing TITAN CORE Codespace

Paste as one command:

```sh
git pull --ff-only && bash review/a53-1360/build-original-preflight-v3.sh
```

When the build reports `BUILT`, download `TITAN-A53-ORIGINAL-PS5-PREFLIGHT-v3.elf` from the Codespaces Explorer. The working production ELF and v2 source/build recipe remain untouched.

## Test and retrieve log

With games closed, inject **v3** once through the same known-working jailbreak and ELF loader that ran v2. Watch for an on-screen `started` notification and a completion notification. Do not enable autoload.

Retrieve the text report from the path displayed in the completion notification; the fallback paths are:

- `/data/titan-a53-original-preflight-v3.txt`
- `/user/data/titan-a53-original-preflight-v3.txt`
- `/system_tmp/titan-a53-original-preflight-v3.txt`

Upload the **text report**, not firmware dumps. Do not change device permissions or authorization IDs to force the test.

## Interpreting the results

- `mp4_dump_lstat` distinguishes an accessible device entry from a missing or unreadable one.
- `mp4_dump_open_nonblock` gives the open-time errno; a failing `access(R_OK)` check does not necessarily establish why `open` failed.
- `kstuff_readiness=confirmed_by_known_check` confirms only its existing readiness syscall; it says nothing about MP4 authorization or A53 execution.
- `kstuff_readiness=not_confirmed` does not necessarily mean kstuff is absent.
- No v3 outcome establishes a successful A53 memory read. The next hardware step requires verified original-PS5 addresses and confirmation of authorization before any bounded SDBGP read can be responsibly attempted.

No PS5 ELF was compiled or tested in the assistant environment; this commit adds the source and build recipe for the user's Codespace.