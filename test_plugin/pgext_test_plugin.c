#include "postgres.h"

#include "fmgr.h"
#include "optimizer/planner.h"

PG_MODULE_MAGIC;

#include "../framework_pgxs/include/pgext.h"

void _PG_init(void) { planner_hook = NULL; }
