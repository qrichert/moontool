CC := clang
CFLAGS := \
	-Wall -Wextra -pedantic -Werror \
	-O3 \
	-std=c17
RM := rm -rf
PREFIX ?= /usr/local

C_FILES := $(wildcard *.c moon/*.c)
C_OBJ_FILES := $(C_FILES:.c=.o)
C_LIBS := -lm


.PHONY: all
all: moontool

.PHONY: moontool
moontool: target/moontool
target/moontool: $(C_OBJ_FILES)
	@mkdir -p target
	$(CC) $(CFLAGS) $^ -o $@ $(C_LIBS)

.PHONY: run
run:
	@./target/moontool

.PHONY: t
t: test
.PHONY: test
test: target/test_moontool
target/test_moontool: tests/test_moon.o
	@mkdir -p target
	$(CC) $(CFLAGS) $^ -o $@ $(C_LIBS)
	@$@
	@$(RM) $@ $^

.PHONY: install
install:
	install -d $(PREFIX)/bin/
	install ./target/moontool $(PREFIX)/bin/

.PHONY: clean
clean:
	$(RM) target/
	$(RM) tests/*.o
	$(RM) $(C_OBJ_FILES)
