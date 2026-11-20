#define _POSIX_C_SOURCE 200809L
/*
 * TITAN CORE original PS5 A53 preflight v3.
 * A basic userland diagnostic; no kernel memory operations, A53 commands,
 * MP4 ioctls, device writes, firmware patching, or native FPKG enablement.
 * Build with ps5-payload-dev/sdk v0.43 through prospero-clang.
 * License: MIT.
 */
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/utsname.h>
#include <unistd.h>

typedef struct {
    unsigned char reserved[45];
    char message[3075];
} notify_request_t;
_Static_assert(sizeof(notify_request_t) == 0xC30, "PS5 notification ABI size");
extern int sceKernelSendNotificationRequest(int device, notify_request_t *req,
                                            size_t size, int blocking);

static void notify(const char *message) {
    notify_request_t request;
    memset(&request, 0, sizeof(request));
    size_t len = strlen(message);
    if (len >= sizeof(request.message)) len = sizeof(request.message) - 1;
    memcpy(request.message, message, len);
    (void)sceKernelSendNotificationRequest(0, &request, sizeof(request), 0);
}

static const char *const log_paths[] = {
    "/data/titan-a53-original-preflight-v3.txt",
    "/user/data/titan-a53-original-preflight-v3.txt",
    "/system_tmp/titan-a53-original-preflight-v3.txt"
};


/* Inspect metadata without accessing device contents or changing its permissions. */
static void inspect_path(FILE *log, const char *label, const char *path) {
    struct stat meta;
    errno = 0;
    if (lstat(path, &meta) == 0) {
        (void)fprintf(log, "%s_lstat=ok;mode=%04o;uid=%lu;gid=%lu;type=%s\n",
            label, (unsigned)meta.st_mode & 07777u,
            (unsigned long)meta.st_uid, (unsigned long)meta.st_gid,
            S_ISDIR(meta.st_mode) ? "dir" :
            S_ISCHR(meta.st_mode) ? "char" :
            S_ISLNK(meta.st_mode) ? "link" : "other");
    } else {
        (void)fprintf(log, "%s_lstat=error;errno=%d\n", label, errno);
    }
    errno = 0;
    if (stat(path, &meta) == 0) {
        (void)fprintf(log, "%s_stat=ok;mode=%04o;uid=%lu;gid=%lu\n",
            label, (unsigned)meta.st_mode & 07777u,
            (unsigned long)meta.st_uid, (unsigned long)meta.st_gid);
    } else {
        (void)fprintf(log, "%s_stat=error;errno=%d\n", label, errno);
    }
    errno = 0;
    if (access(path, F_OK) == 0) {
        (void)fprintf(log, "%s_access_exists=ok\n", label);
    } else {
        (void)fprintf(log, "%s_access_exists=error;errno=%d\n", label, errno);
    }
    errno = 0;
    if (access(path, R_OK) == 0) {
        (void)fprintf(log, "%s_access_read=ok\n", label);
    } else {
        (void)fprintf(log, "%s_access_read=error;errno=%d\n", label, errno);
    }
}

/* TITAN CORE src/stack.rs uses this side-effect-free KEKCALL_CHECK.
 * The result cannot prove MP4 authorization or A53 code execution.
 */
static void report_kstuff_readiness(FILE *log) {
#if defined(__x86_64__)
    uint64_t result = UINT64_C(0xffffffff00000027);
    uint64_t zero = 0;
    unsigned char carry = 1;
    register uint64_t arg4 __asm__("r10") = 0;
    register uint64_t arg5 __asm__("r8") = 0;
    register uint64_t arg6 __asm__("r9") = 0;
    __asm__ volatile(
        "syscall\n\tsetc %1"
        : "+a"(result), "=q"(carry), "+r"(arg4), "+r"(arg5), "+r"(arg6)
        : "D"(zero), "S"(zero), "d"(zero)
        : "rcx", "r11", "cc", "memory");
    (void)fprintf(log, "kstuff_probe=returned;carry=%u;rax=0x%llx\n",
        (unsigned)carry, (unsigned long long)result);
    (void)fprintf(log, "kstuff_readiness=%s\n",
        (carry == 0 && result == 0) ? "confirmed_by_known_check" : "not_confirmed");
#else
    (void)fprintf(log, "kstuff_readiness=unsupported_architecture\n");
#endif
}

int main(void) {
    notify("TITAN A53 preflight v3: started");
    FILE *log = NULL;
    const char *location = NULL;
    int last_errno = 0;
    for (size_t i = 0; i < sizeof(log_paths) / sizeof(log_paths[0]); ++i) {
        errno = 0;
        log = fopen(log_paths[i], "w");
        if (log != NULL) { location = log_paths[i]; break; }
        last_errno = errno;
    }
    if (log == NULL) {
        char error[140];
        (void)snprintf(error, sizeof(error),
            "TITAN A53 preflight v3: unable to create log (errno %d)", last_errno);
        notify(error);
        return 2;
    }
    /* Set an early marker so a crash later can be distinguished from startup. */
    (void)fprintf(log, "TITAN CORE original PS5 preflight v3 started\n");
    (void)fprintf(log, "log_path=%s\n", location);
    (void)fprintf(log, "mode=read_only_userland_and_kstuff_readiness_check\n");
    (void)fprintf(log, "a53_execution_verified=false\n");
    (void)fprintf(log, "native_fpkg_enabled=false\n");
    (void)fflush(log);
    (void)fsync(fileno(log));

    struct utsname uts;
    errno = 0;
    if (uname(&uts) == 0) {
        (void)fprintf(log, "sysname=%s\nrelease=%s\nmachine=%s\n",
            uts.sysname, uts.release, uts.machine);
    } else {
        (void)fprintf(log, "uname_errno=%d\n", errno);
    }
    (void)fprintf(log, "process_pid=%ld\nuid=%lu\neuid=%lu\ngid=%lu\negid=%lu\n",
        (long)getpid(), (unsigned long)getuid(), (unsigned long)geteuid(),
        (unsigned long)getgid(), (unsigned long)getegid());
    gid_t groups[16];
    errno = 0;
    int group_count = getgroups(16, groups);
    if (group_count >= 0) {
        (void)fprintf(log, "supplementary_groups=%d\n", group_count);
        for (int i = 0; i < group_count; ++i) {
            (void)fprintf(log, "supplementary_group_%d=%lu\n",
                i, (unsigned long)groups[i]);
        }
    } else {
        (void)fprintf(log, "supplementary_groups=error;errno=%d\n", errno);
    }

    inspect_path(log, "dev", "/dev");
    inspect_path(log, "mp4_dir", "/dev/mp4");
    inspect_path(log, "mp4_dump", "/dev/mp4/dump");

    errno = 0;
    int fd = open("/dev/mp4/dump", O_RDONLY | O_NONBLOCK);
    if (fd >= 0) {
        (void)fprintf(log, "mp4_dump_open_nonblock=ok\n");
        (void)close(fd);
    } else {
        (void)fprintf(log, "mp4_dump_open_nonblock=error;errno=%d\n", errno);
    }
    report_kstuff_readiness(log);
    (void)fprintf(log, "authorization_modified=false\n");
    (void)fprintf(log, "mp4_ioctl_performed=false\n");
    (void)fprintf(log, "a53_memory_read_performed=false\n");
    (void)fprintf(log, "result=complete; this does not demonstrate A53 execution\n");
    (void)fflush(log);
    (void)fsync(fileno(log));
    (void)fclose(log);

    char completed[250];
    (void)snprintf(completed, sizeof(completed),
        "TITAN A53 preflight v3: complete; log at %s", location);
    notify(completed);
    return 0;
}
