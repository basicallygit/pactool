.PHONY: all clean install

TARGET = pactool
INSTALL_PATH = /usr/bin

all:
	cargo build --release

clean:
	cargo clean

install:
	install -Dm755 target/release/$(TARGET) $(INSTALL_PATH)/$(TARGET)
