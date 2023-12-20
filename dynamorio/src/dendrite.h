
#include "dr_defines.h"
#include <stdint.h>

typedef enum _branch_kind {
	COND_BRANCH   = 0x10,
	JMP_DIRECT    = 0x20,
	JMP_INDIRECT  = 0x21,
	CALL_DIRECT   = 0x40,
	CALL_INDIRECT = 0x41,
	RETURN        = 0x81,
} branch_kind;

typedef struct _trace_record {
	// Program counter value
	uint64 pc;
	// Target address
	uint64 tgt;
	// Outcome (taken=1, not-taken=0)
	uint32_t outcome;
	// Branch kind
	uint32_t kind;
} trace_record;
