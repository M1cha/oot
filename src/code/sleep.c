#include "global.h"
#include <unistd.h>
#include <errno.h>
#include <time.h>

// TODO: use clock_nanosleep
void Sleep_Nsec(u32 nsec) {
    struct timespec ts;
    struct timespec ts_rem;
    int ret;

    ts.tv_sec = 0;
    ts.tv_nsec = nsec;

    while (ts.tv_nsec) {
        ret = nanosleep(&ts, &ts_rem);
        if (ret < 0) {
            if (errno == EINTR) {
                ts = ts_rem;
                continue;
            }

            abort();
        }

        break;
    }
}

void Sleep_Usec(u32 usec) {
    usleep(usec);
}

// originally "msleep"
void Sleep_Msec(u32 ms) {
    Sleep_Usec(ms * 1000);
}

void Sleep_Sec(u32 sec) {
    sleep(sec);
}
