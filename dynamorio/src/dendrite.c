#include "dr_api.h"
#include "drmgr.h"
#include "drx.h"
#include "dendrite.h"

static client_id_t client_id;
static int tls_log_idx;

static void 
hook_conditional_branch(app_pc pc, app_pc tgt, int taken)
{
    void *drcontext = dr_get_current_drcontext();
    uint32_t ilen = 0;
	uint32_t flags = ILEN_FIELD(ilen) | BRN_FLAG | TAKEN_FIELD(taken);
	trace_record record = {
		.pc = (uint64)pc,
		.tgt = (uint64)tgt,
		.flags = flags,
	};

    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx);
	dr_write_file(log, &record, sizeof(trace_record));
}

static void hook_jump_direct(app_pc pc, app_pc tgt)
{
    void *drcontext = dr_get_current_drcontext();
    uint32_t ilen = 0;
	uint32_t flags = ILEN_FIELD(ilen) | JMP_FLAG | TAKEN_FLAG;
	trace_record record = {
		.pc = (uint64)pc,
		.tgt = (uint64)tgt,
		.flags = flags,
	};

    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx);
	dr_write_file(log, &record, sizeof(trace_record));
}

static void hook_jump_indirect(app_pc pc, app_pc tgt)
{
    void *drcontext = dr_get_current_drcontext();
    uint32_t ilen = 0;
	uint32_t flags = ILEN_FIELD(ilen) | JMP_FLAG | IND_FLAG | TAKEN_FLAG;
	trace_record record = {
		.pc = (uint64)pc,
		.tgt = (uint64)tgt,
		.flags = flags,
	};

    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx);
	dr_write_file(log, &record, sizeof(trace_record));
}

static void hook_ret(app_pc pc, app_pc tgt)
{
    void *drcontext = dr_get_current_drcontext();
    uint32_t ilen = 0;
	uint32_t flags = ILEN_FIELD(ilen) | RET_FLAG | IND_FLAG | TAKEN_FLAG;
	trace_record record = {
		.pc = (uint64)pc,
		.tgt = (uint64)tgt,
		.flags = flags,
	};

    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx);
	dr_write_file(log, &record, sizeof(trace_record));
}

static void hook_call_direct(app_pc pc, app_pc tgt)
{
    void *drcontext = dr_get_current_drcontext();
    uint32_t ilen = 0;
	uint32_t flags = ILEN_FIELD(ilen) | CALL_FLAG | TAKEN_FLAG;
	trace_record record = {
		.pc = (uint64)pc,
		.tgt = (uint64)tgt,
		.flags = flags,
	};

    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx);
	dr_write_file(log, &record, sizeof(trace_record));
}

static void hook_call_indir(app_pc pc, app_pc tgt)
{
    void *drcontext = dr_get_current_drcontext();
    uint32_t ilen = 0;
	uint32_t flags = ILEN_FIELD(ilen) | CALL_FLAG | IND_FLAG | TAKEN_FLAG;
	trace_record record = {
		.pc = (uint64)pc,
		.tgt = (uint64)tgt,
		.flags = flags,
	};

    file_t log = (file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx);
	dr_write_file(log, &record, sizeof(trace_record));
}

static dr_emit_flags_t
event_bb_analysis(void *drcontext, void *tag, instrlist_t *bb, bool for_trace,
		bool translating, void **user_data)
{
	// NOTE: 'user_data' is apparently passed to the next stage
	//// Iterate over all instructions in the block 
	//for (instr = instrlist_first(bb), num_instrs = 0; instr != NULL; 
	//		instr = instr_get_next(instr)) 
	//{
	//	num_instrs++;
	//}
	//*user_data = (void*)instr_lens;
	return DR_EMIT_DEFAULT; 
}

// NOTE: This is called for each instruction in a basic block. 
// Inserts a hook that will capture info about a control-flow instruction.
static dr_emit_flags_t
event_app_instruction(void *drcontext, void *tag, instrlist_t *bb, 
	instr_t *instr, bool for_trace, bool translating, void *user_data)
{
	// If this is a control-flow instruction, insert a hook that will record
	// relevant info about the instruction
	if (instr_is_cti(instr)) {
		
		// NOTE: How do we pass this through to the cleancalls? 
		int instr_len = instr_length(drcontext, instr);

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
		"/tmp", dr_get_process_id(), "dendrite", "bin", 
		DR_FILE_ALLOW_LARGE, buf, sizeof(buf) / sizeof((buf)[0])
	);
    DR_ASSERT(log != INVALID_FILE);
    drmgr_set_tls_field(drcontext, tls_log_idx, (void *)(ptr_uint_t)log);
}

static void event_thread_exit(void *drcontext)
{
	dr_close_file((file_t)(ptr_uint_t)drmgr_get_tls_field(drcontext, tls_log_idx));
}

static void event_exit(void)
{
    dr_log(NULL, DR_LOG_ALL, 1, "Client 'dendrite' exiting");
    if (!drmgr_unregister_bb_insertion_event(event_app_instruction) ||
        !drmgr_unregister_tls_field(tls_log_idx))
        DR_ASSERT(false);
    drmgr_exit();
}

DR_EXPORT void dr_client_main(client_id_t id, int argc, const char *argv[])
{
    dr_set_client_name("dendrite", "https://github.com/eigenform/dendrite");
    drmgr_init();
    client_id = id;
    tls_log_idx = drmgr_register_tls_field();
    dr_register_exit_event(event_exit);
    if (!drmgr_register_thread_init_event(event_thread_init) ||
        !drmgr_register_thread_exit_event(event_thread_exit) ||
        !drmgr_register_bb_instrumentation_event(
			event_bb_analysis, event_app_instruction, NULL))
	{
        DR_ASSERT(false);
	}
}

