CC = $(ARCH)-linux-musl-gcc
CFLAGS = -static

TARGET_DIR = build/$(ARCH)
TARGET = $(TARGET_DIR)/hello

all: build

prepare:
	mkdir -p $(TARGET_DIR)

build: prepare $(TARGET)

$(TARGET): hello.c
	$(CC) $(CFLAGS) -o $@ $^

clean:
	rm -f $(TARGET)

.PHONY: all prepare build clean
