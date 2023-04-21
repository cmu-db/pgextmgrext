void _PG_init__pgext(void); // The original PG_init entry point

// begin pgext APIs
extern PGDLLIMPORT void __attribute__((weak))
__pgext_before_init(const char *name);
extern PGDLLIMPORT void __attribute__((weak)) __pgext_after_init();
// end pgext APIs

void _PG_init() {
  __pgext_before_init("pgext_test_plugin");
  _PG_init__pgext();
  __pgext_after_init();
}

// Redefine the original PG_init entry point to `_PG_init__pgext`
#define _PG_init _PG_init__pgext
