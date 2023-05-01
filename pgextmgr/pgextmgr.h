#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


typedef struct String String;

typedef bool (*OutputRewriterFilter)(QueryDesc *query_desc);

typedef void *(*OutputRewriterStartup)(int operation, TupleDesc type_info);

typedef void (*OutputRewriterShutdown)(void*);

typedef void (*OutputRewriterDestroy)(void*);

typedef bool (*OutputRewriterReceiveSlot)(void*, TupleTableSlot *slot, void*, bool(*)(void*));

typedef struct OutputRewriter {
  OutputRewriterFilter filter;
  OutputRewriterStartup startup;
  OutputRewriterShutdown shutdown;
  OutputRewriterDestroy destroy;
  OutputRewriterReceiveSlot receive_slot;
} OutputRewriter;

typedef struct PgExtApi {
  const struct String *plugin;
  void (*register_output_rewriter)(const struct PgExtApi *api, const struct OutputRewriter *rewriter);
} PgExtApi;

void __pgext_after_init(void);

struct PgExtApi *__pgext_before_init(const char *name);
