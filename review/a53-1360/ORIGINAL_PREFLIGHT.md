# Original PS5 A53 preflight v2

This is a corrected **original PS5** userland diagnostic, not an A53 exploit or native FPKG payload. It performs no A53 read/write, kernel memory access, or MP4 ioctl. It only queries basic system info and tries a read-only open of `/dev/mp4/dump`. The previous freestanding/static-PIE prototype may not have been loadable by the PS5 ELF loader; the lack of its report cannot establish a specific cause.

## Build on iPhone using existing TITAN CORE Codespaces

Copy the following **one line** into the Codespaces terminal, then press Return:

```sh
git pull --ff-only && bash review/a53-1360/build-original-preflight.sh
```

The script builds with the exact PS5 Payload SDK Docker image used by TITAN CORE, but doesn't touch the production `titan-core.elf` or bundled kstuff/ShadowMount payloads. After a successful build, download `TITAN-A53-ORIGINAL-PS5-PREFLIGHT-v2.elf` from the repository root in the Codespaces Explorer.

## Console test

1. Stop running games. Use an already working jailbreak and ELF loader to send the v2 ELF once; do not autoload it.
2. Watch for the on-screen notification `TITAN A53 preflight v2: started`, followed by either a log-path success notification or a log error notification.
3. When a log file can be written, the notification displays its actual location. The fallback search order is:
   - `/data/titan-a53-original-preflight.txt`
   - `/user/data/titan-a53-original-preflight.txt`
   - `/system_tmp/titan-a53-original-preflight.txt` (temporary across reboot)
4. Retrieve the file with the PS5 file manager or FTP while the console is running.

**Interpretation:** No `started` notification means the payload may not have loaded, or notifications were unavailable. A `started` notification with a write error means it executed but could not open any of the file paths. A completed message means it reached the end of diagnostics. Successful open of `/dev/mp4/dump` demonstrates only basic device access, **not** A53 readback/execution, PPR patching, or native FPKG support.

The firmware/model identity is not attested. The ELF is not tested on a physical PS5 and cannot guarantee console compatibility, despite using the SDK.
