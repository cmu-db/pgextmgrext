MODULES = pgext_test_plugin
EXTENSION = pgext_test_plugin
DATA = pgext_test_plugin--0.0.1.sql
OBJS = pgext_test_plugin.o

PG_LDFLAGS = -Wl,-U,___pgext_before_init,-U,___pgext_after_init

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)
