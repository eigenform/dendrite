#include "pin.H"
#include <iostream>
#include <fstream>
#include <cstdint>

using std::cerr;
using std::endl;
using std::string;

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
#define TAKEN_FIELD(x) ((x ? 1 : 0) << 5)
#define ILEN_FIELD(x)  ((x & 0xf) << 28)

typedef struct _trace_record {
	// Program counter value
	uint64_t pc;
	// Target address
	uint64_t tgt;
	// Flags
	uint32_t flags;
} trace_record;


std::ostream* out;

KNOB< string > KnobOutputFile(KNOB_MODE_WRITEONCE, "pintool", "o", "/tmp/trace.bin", 
		"output binary trace filename");

//KNOB< BOOL > KnobCount(KNOB_MODE_WRITEONCE, "pintool", "count", "1",
//		"count instructions, basic blocks and threads in the application");


//VOID Trace(TRACE trace, VOID* v)
//{
//    for (BBL bbl = TRACE_BblHead(trace); BBL_Valid(bbl); bbl = BBL_Next(bbl)) {
//    }
//}


VOID fini(INT32 code, VOID* v)
{
}

// This is the function called when we actually evaluate the program 
static VOID write_trace(ADDRINT ip, ADDRINT target, BOOL taken, UINT32 flags)
{
	trace_record record = {
		.pc = (uint64_t)ip,
		.tgt = (uint64_t)target,
		.flags = (uint32_t)(flags | TAKEN_FIELD(taken)),
	};

	// NOTE: This is probably very slow ...
	out->write(reinterpret_cast<const char*>(&record), sizeof(trace_record));
	out->flush();
}


VOID instrument(INS ins, VOID *v)
{
	UINT32 ilen = 0;
	UINT32 flags = 0;

	if (INS_IsControlFlow(ins)) {
		ilen = INS_Size(ins);
		flags |= ILEN_FIELD(ilen);

		if (INS_HasFallThrough(ins)) {
			flags |= BRN_FLAG;
		} else {
			if (INS_IsCall(ins)) {
				flags |= CALL_FLAG;
			}
			else if (INS_IsRet(ins)) {
				flags |= RET_FLAG;
			} else {
				flags |= JMP_FLAG;
			}
		}

		if (INS_IsIndirectControlFlow(ins)) {
			flags |= IND_FLAG;
		}

		INS_InsertCall(ins, IPOINT_BEFORE, (AFUNPTR)write_trace, 
			IARG_INST_PTR,
			IARG_BRANCH_TARGET_ADDR,
			IARG_BRANCH_TAKEN,
			IARG_UINT32, flags,
			IARG_END
		);
	}

}

int main(int argc, char* argv[])
{
    if (PIN_Init(argc, argv)) {
		fprintf(stderr, "%s\n", KNOB_BASE::StringKnobSummary().c_str());
		return -1;
    }

    string fileName = KnobOutputFile.Value();
    if (!fileName.empty()) {
        out = new std::ofstream(fileName.c_str());
    }

    //TRACE_AddInstrumentFunction(Trace, 0);
	INS_AddInstrumentFunction(instrument, 0);
    PIN_AddFiniFunction(fini, 0);

    PIN_StartProgram();
    return 0;
}

