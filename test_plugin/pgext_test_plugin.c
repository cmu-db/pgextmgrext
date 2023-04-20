#include "postgres.h"

#include "fmgr.h"
#include "optimizer/planner.h"

PG_MODULE_MAGIC;

#include "../framework_pgxs/include/pgext.h"

static planner_hook_type prev_planner = NULL;

static PlannedStmt *test_plugin_planner_hook(Query *parse,
                                             const char *query_string,
                                             int cursorOptions,
                                             ParamListInfo boundParams) {
  if (prev_planner) {
    return prev_planner(parse, query_string, cursorOptions, boundParams);
  } else {
    return standard_planner(parse, query_string, cursorOptions, boundParams);
  }
}
void _PG_init(void) {
  prev_planner = planner_hook;
  planner_hook = test_plugin_planner_hook;
}
