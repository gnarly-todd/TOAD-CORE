#define _POSIX_C_SOURCE 200809L
/*
 * TITAN CORE original PS5 A53 preflight v2.
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
    "/data/titan-a53-original-preflight.txt",
    "/user/data/titan-a53-original-preflight.txt",
    "/system_tmp/titan-a53-original-preflight.txt"
};

int main(void) {
    notify("TITAN A53 preflight v2: started");
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
            "TITAN A53 preflight v2: unable to create log (errno %d)", last_errno);
        notify(error);
        return 2;
    }
    /* Set an early marker so a crash later can be distinguished from startup. */
    (void)fprintf(log, "TITAN CORE original PS5 preflight v2 started\n");
    (void)fprintf(log, "log_path=%s\n", location);
    (void)fprintf(log, "mode=read_only_userland\n");
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
    errno = 0;
    int fd = open("/dev/mp4/dump", O_RDONLY);
    if (fd >= 0) {
        (void)fprintf(log, "mp4_dump=open_readonly_ok\n");
        (void)close(fd);
    } else {
        (void)fprintf(log, "mp4_dump=open_readonly_failed;errno=%d\n", errno);
    }
    (void)fprintf(log, "mp4_ioctl_performed=false\n");
    (void)fprintf(log, "a53_memory_read_performed=false\n");
    (void)fprintf(log, "result=complete; this does not demonstrate A53 execution\n");
    (void)fflush(log);
    (void)fsync(fileno(log));
    (void)fclose(log);

    char completed[250];
    (void)snprintf(completed, sizeof(completed),
        "TITAN A53 preflight v2: complete; log at %s", location);
    notify(completed);
    return 0;
}
