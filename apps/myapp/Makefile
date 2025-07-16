# CC = $(ARCH)-linux-musl-gcc

CC = ~/Downloads/musl-test/bin/x86_64-linux-musl-gcc
CFLAGS = -static -g

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
