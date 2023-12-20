#include "dr_api.h"
#include "drmgr.h"
#include "utils.h"

/* This is mostly reproduced from 'api/samples/cbrtrace.c' in dynamorio.
 */

static void
event_exit(void);

DR_EXPORT void
dr_client_main(client_id_t id, int argc, const char *argv[])
{
    /* empty client */
    dr_set_client_name("dendrite", "https://github.com/eigenform/dendrite");
    dr_register_exit_event(event_exit);
}

static void
event_exit(void)
{
    /* empty client */
}
