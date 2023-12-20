#include "dr_api.h"
#include "drmgr.h"
#include "drx.h"

static client_id_t client_id;
static int tls_idx;

static void 
hook_conditional_branch(app_pc inst_addr, app_pc targ_addr, int taken)
{
    void *drcontext = dr_get_current_drcontext();
    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx);
    dr_fprintf(log, "cond_brn      pc=%p tgt=%p %s\n",
		inst_addr, targ_addr, taken == 0 ? "N" : "T");
}

static void hook_jump_direct(app_pc inst_addr, app_pc targ_addr)
{
    void *drcontext = dr_get_current_drcontext();
    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx);
    dr_fprintf(log, "direct_jump   pc=%p tgt=%p\n", inst_addr, targ_addr);
}

static void hook_jump_indirect(app_pc inst_addr, app_pc targ_addr)
{
    void *drcontext = dr_get_current_drcontext();
    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx);
    dr_fprintf(log, "indirect_jump pc=%p tgt=%p\n", inst_addr, targ_addr);
}

static void hook_ret(app_pc inst_addr, app_pc targ_addr)
{
    void *drcontext = dr_get_current_drcontext();
    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx);
    dr_fprintf(log, "return        pc=%p tgt=%p\n", inst_addr, targ_addr);
}

static void hook_call_direct(app_pc inst_addr, app_pc targ_addr)
{
    void *drcontext = dr_get_current_drcontext();
    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx);
    dr_fprintf(log, "direct_call   pc=%p tgt=%p\n", inst_addr, targ_addr);
}

static void hook_call_indir(app_pc inst_addr, app_pc targ_addr)
{
    void *drcontext = dr_get_current_drcontext();
    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx);
    dr_fprintf(log, "indirect_call pc=%p tgt=%p\n", inst_addr, targ_addr);
}

static dr_emit_flags_t
event_app_instruction(void *drcontext, void *tag, instrlist_t *bb, 
	instr_t *instr, bool for_trace, bool translating, void *user_data)
{
	if (instr_is_cti(instr)) {
		// Conditional branch
		if (instr_is_cbr(instr)) {
			dr_insert_cbr_instrumentation(drcontext, bb, instr, 
				(void *)hook_conditional_branch);
		}
		// Direct jump
		else if (instr_is_ubr(instr)) {
			dr_insert_ubr_instrumentation(drcontext, bb, instr,
				(void *)hook_jump_direct);
		}
		// Direct call
		else if (instr_is_call_direct(instr)) {
			dr_insert_call_instrumentation(drcontext, bb, instr,
				(void *)hook_call_direct);
		}
		// Return 
		else if (instr_is_return(instr)) {
			dr_insert_mbr_instrumentation(drcontext, bb, instr,
				(void *)hook_ret, SPILL_SLOT_1);
		}
		// Indirect call
		else if (instr_is_call_indirect(instr)) {
			dr_insert_mbr_instrumentation(drcontext, bb, instr,
				(void *)hook_call_indir, SPILL_SLOT_1);
		}
		// Indirect jump
		else if (instr_is_mbr(instr)) {
			dr_insert_mbr_instrumentation(drcontext, bb, instr,
				(void *)hook_jump_indirect, SPILL_SLOT_1);
		}
	}
    return DR_EMIT_DEFAULT;
}

static void event_thread_init(void *drcontext)
{
    file_t log;
	char buf[256];
	size_t len;
	log = drx_open_unique_appid_file(
		"/tmp", dr_get_process_id(), "dendrite", "log", 
		DR_FILE_ALLOW_LARGE, buf, sizeof(buf) / sizeof((buf)[0])
	);
    DR_ASSERT(log != INVALID_FILE);
    drmgr_set_tls_field(drcontext, tls_idx, (void *)(ptr_uint_t)log);
}

static void event_thread_exit(void *drcontext)
{
	dr_close_file((file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_idx));
}

static void event_exit(void)
{
    dr_log(NULL, DR_LOG_ALL, 1, "Client 'dendrite' exiting");
    if (!drmgr_unregister_bb_insertion_event(event_app_instruction) ||
        !drmgr_unregister_tls_field(tls_idx))
        DR_ASSERT(false);
    drmgr_exit();
}

DR_EXPORT void dr_client_main(client_id_t id, int argc, const char *argv[])
{
    dr_set_client_name("dendrite", "https://github.com/eigenform/dendrite");
    drmgr_init();
    client_id = id;
    tls_idx = drmgr_register_tls_field();
    dr_register_exit_event(event_exit);
    if (!drmgr_register_thread_init_event(event_thread_init) ||
        !drmgr_register_thread_exit_event(event_thread_exit) ||
        !drmgr_register_bb_instrumentation_event(NULL, event_app_instruction, NULL))
        DR_ASSERT(false);
}

