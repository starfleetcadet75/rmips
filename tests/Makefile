CC = mips-linux-gnu-gcc
LD = mips-linux-gnu-ld
OBJCOPY = mips-linux-gnu-objcopy
CFLAGS = -Wall -EL -fno-pic -mno-abicalls -march=3000 -mfp32

SRC_DIR = src
BUILD_DIR = build

.PHONY: all clean

all: emptymain.rom

emptymain.rom: emptymain.bin
	$(OBJCOPY) -O binary $(BUILD_DIR)/emptymain.bin $(BUILD_DIR)/emptymain.rom

emptymain.bin: emptymain.o setup.o
	$(LD) -EL -T ld.script -o $(BUILD_DIR)/emptymain.bin $(BUILD_DIR)/setup.o $(BUILD_DIR)/emptymain.o

emptymain.o: $(SRC_DIR)/emptymain.c
	$(CC) $(CFLAGS) -o $(BUILD_DIR)/emptymain.o -c $(SRC_DIR)/emptymain.c

setup.o: $(SRC_DIR)/setup.s
	$(CC) $(CFLAGS) -c -x assembler-with-cpp -o $(BUILD_DIR)/setup.o $(SRC_DIR)/setup.s

clean:
	rm -f $(BUILD_DIR)/*.o $(BUILD_DIR)/*.bin $(BUILD_DIR)/*.rom
