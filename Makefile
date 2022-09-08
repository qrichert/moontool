CC := clang
CFLAGS := \
	-Wall -Wextra -pedantic -Werror \
	-O3 \
	-std=c17
CXX := clang++
CXXFLAGS := \
	-Wall -Wextra -pedantic -Werror \
	-O3 \
	-std=c++20
RM := rm -rf
PREFIX ?= /usr/local

C_FILES := $(wildcard *.c moon/*.c)
C_OBJ_FILES := $(C_FILES:.c=.o)
C_LIBS := -lm
CPP_FILES := $(wildcard *.cpp moon/*.cpp)
CPP_OBJ_FILES := $(CPP_FILES:.cpp=.o)


.PHONY: all
all: moontool

.PHONY: moontool
moontool: build/moontool
build/moontool: $(CPP_OBJ_FILES) $(C_OBJ_FILES)
	@mkdir -p build
	$(CXX) $(CXXFLAGS) $^ -o $@ $(C_LIBS)


.PHONY: install
install:
	install -d $(PREFIX)/bin/
	install ./build/moontool $(PREFIX)/bin/

.PHONY: clean
clean:
	$(RM) build
	$(RM) $(C_OBJ_FILES)
	$(RM) $(CPP_OBJ_FILES)
