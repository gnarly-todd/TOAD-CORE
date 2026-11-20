/* Payload entry, linked in front of Rust's main with -Wl,--wrap=main.
 * elfldr gives raw ELF streams a generic name, and file-backed loads inherit
 * the supplied filename. Set the PS5 thread/process listing name ourselves
 * so the actual active-process entry is consistently "toad-core.elf".
 *
 * This uses the same SYS_thr_set_name(-1, ...) as PS5 Payload Manager.
 * Failure to rename must not prevent the existing payload from starting.
 * Do not rename the internal saved-copy path or legacy data directory.
 */
#include <sys/syscall.h>
#include <unistd.h>

extern int __real_main(int argc, char **argv);

int __wrap_main(int argc, char **argv) {
    /* The 13-character name fits PS5's 20-byte ki_comm field. */
    (void)syscall(SYS_thr_set_name, -1, "toad-core.elf");
    return __real_main(argc, argv);
}
