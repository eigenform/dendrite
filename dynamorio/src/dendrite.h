
#include "dr_defines.h"
#include <stdint.h>

// Branch instruction
#define BRN_FLAG   (1 << 0)
// Jump instruction
#define JMP_FLAG   (1 << 1)
// Call instruction
#define CALL_FLAG  (1 << 2)
// Return instruction
#define RET_FLAG   (1 << 3)
// Indirect addressing
#define IND_FLAG   (1 << 4)
// Taken outcome
#define TAKEN_FLAG (1 << 5)

#define TAKEN_FIELD(x) ((x == 0 ? 1 : 0) << 5)
#define ILEN_FIELD(x) ((x & 0xf) << 28)

typedef struct _trace_record {
	// Program counter value
	uint64 pc;
	// Target address
	uint64 tgt;
	// Flags
	uint32_t flags;
} trace_record;
