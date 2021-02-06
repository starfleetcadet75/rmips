CC = mips-linux-gnu-gcc
LD = mips-linux-gnu-ld
OBJCOPY = mips-linux-gnu-objcopy
CFLAGS = -Wall -g -fno-pic -mno-abicalls -march=3000 -mfp32

SRC_DIR = src
BUILD_DIR = build

.PHONY: all clean

all: emptymain_le.rom emptymain_be.rom

emptymain_le.rom: emptymain_le.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/emptymain_le.elf $(BUILD_DIR)/emptymain_le.rom

emptymain_be.rom: emptymain_be.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/emptymain_be.elf $(BUILD_DIR)/emptymain_be.rom

emptymain_le.elf: emptymain_le.o setup_le.o
	$(LD) -EL -T ld.script -o $(BUILD_DIR)/emptymain_le.elf $(BUILD_DIR)/setup_le.o $(BUILD_DIR)/emptymain_le.o

emptymain_be.elf: emptymain_be.o setup_be.o
	$(LD) -EB -T ld.script -o $(BUILD_DIR)/emptymain_be.elf $(BUILD_DIR)/setup_be.o $(BUILD_DIR)/emptymain_be.o

emptymain_le.o: $(SRC_DIR)/emptymain.c
	$(CC) $(CFLAGS) -EL -o $(BUILD_DIR)/emptymain_le.o -c $(SRC_DIR)/emptymain.c

emptymain_be.o: $(SRC_DIR)/emptymain.c
	$(CC) $(CFLAGS) -EB -o $(BUILD_DIR)/emptymain_be.o -c $(SRC_DIR)/emptymain.c

setup_le.o: $(SRC_DIR)/setup.s
	$(CC) $(CFLAGS) -EL -c -x assembler-with-cpp -o $(BUILD_DIR)/setup_le.o $(SRC_DIR)/setup.s

setup_be.o: $(SRC_DIR)/setup.s
	$(CC) $(CFLAGS) -EB -c -x assembler-with-cpp -o $(BUILD_DIR)/setup_be.o $(SRC_DIR)/setup.s

clean:
	rm -f $(BUILD_DIR)/*.o $(BUILD_DIR)/*.elf $(BUILD_DIR)/*.rom
