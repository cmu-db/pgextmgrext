#include "postgres.h"

#include "access/detoast.h"
#include "executor/executor.h"
#include "fmgr.h"
#include "optimizer/planner.h"
#include "tcop/dest.h"
#include "utils/lsyscache.h"
#include "utils/palloc.h"

#include "pgextmgr.h"

PG_MODULE_MAGIC;

static struct PgExtApi *api;

static bool poopReceiveSlot(void *self, TupleTableSlot *slot, void *ctx,
                            bool (*cb)(void *)) {
  TupleDesc typeinfo = slot->tts_tupleDescriptor;
  int natts = typeinfo->natts;
  bytea **bb = palloc_array(bytea *, natts);

  for (int i = 0; i < natts; ++i) {
    bb[i] = NULL;
    Oid ty = TupleDescAttr(typeinfo, i)->atttypid;
    if (ty != VARCHAROID && ty != TEXTOID) {
      continue;
    }
    bool isnull;
    Datum attr = slot_getattr(slot, i + 1, &isnull);
    int64 vallen = toast_raw_datum_size(attr) - VARHDRSZ;

    const char *poop_emoji = "\360\237\222\251";
    const int poop_emoji_len = 4;
    int64 bytea_len = vallen * poop_emoji_len + VARHDRSZ;
    bytea *b = (bytea *)palloc(bytea_len);
    SET_VARSIZE(b, bytea_len);
    for (int j = 0; j < vallen; j += 1) {
      memcpy(VARDATA(b) + j * poop_emoji_len, poop_emoji, poop_emoji_len);
    }
    bb[i] = b;

    slot->tts_values[i] = PointerGetDatum(b);
  }

  bool res = cb(ctx);

  for (int i = 0; i < natts; ++i) {
    if (bb[i]) {
      pfree(bb[i]);
    }
  }

  pfree(bb);

  return res;
}

void _PG_init(void) {
  api = __pgext_before_init("pgext_pg_poop");
  OutputRewriter r;
  r.destroy = NULL;
  r.startup = NULL;
  r.filter = NULL;
  r.shutdown = NULL;
  r.receive_slot = poopReceiveSlot;
  api->register_output_rewriter(api, &r);
  __pgext_after_init();
}
