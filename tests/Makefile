CC = mips-linux-gnu-gcc
LD = mips-linux-gnu-ld
OBJCOPY = mips-linux-gnu-objcopy
CFLAGS = -Wall -g -fno-pic -mno-abicalls -march=3000 -mfp32

SRC_DIR = src
BUILD_DIR = build

.PHONY: all clean

all: arithmetic.rom bitwise.rom branch.rom logic.rom memory.rom

arithmetic.rom: arithmetic.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/arithmetic.elf $(BUILD_DIR)/arithmetic.rom

bitwise.rom: bitwise.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/bitwise.elf $(BUILD_DIR)/bitwise.rom

branch.rom: branch.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/branch.elf $(BUILD_DIR)/branch.rom

logic.rom: logic.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/logic.elf $(BUILD_DIR)/logic.rom

memory.rom: memory.elf
	$(OBJCOPY) -O binary $(BUILD_DIR)/memory.elf $(BUILD_DIR)/memory.rom

arithmetic.elf: arithmetic.o
	$(LD) -EL -T link.ld -o $(BUILD_DIR)/arithmetic.elf $(BUILD_DIR)/arithmetic.o

bitwise.elf: bitwise.o
	$(LD) -EL -T link.ld -o $(BUILD_DIR)/bitwise.elf $(BUILD_DIR)/bitwise.o

branch.elf: branch.o
	$(LD) -EL -T link.ld -o $(BUILD_DIR)/branch.elf $(BUILD_DIR)/branch.o

logic.elf: logic.o
	$(LD) -EL -T link.ld -o $(BUILD_DIR)/logic.elf $(BUILD_DIR)/logic.o

memory.elf: memory.o
	$(LD) -EL -T link.ld -o $(BUILD_DIR)/memory.elf $(BUILD_DIR)/memory.o

arithmetic.o: $(SRC_DIR)/arithmetic.s
	$(CC) $(CFLAGS) -EL -c -x assembler-with-cpp -o $(BUILD_DIR)/arithmetic.o $(SRC_DIR)/arithmetic.s

bitwise.o: $(SRC_DIR)/bitwise.s
	$(CC) $(CFLAGS) -EL -c -x assembler-with-cpp -o $(BUILD_DIR)/bitwise.o $(SRC_DIR)/bitwise.s

branch.o: $(SRC_DIR)/branch.s
	$(CC) $(CFLAGS) -EL -c -x assembler-with-cpp -o $(BUILD_DIR)/branch.o $(SRC_DIR)/branch.s

logic.o: $(SRC_DIR)/logic.s
	$(CC) $(CFLAGS) -EL -c -x assembler-with-cpp -o $(BUILD_DIR)/logic.o $(SRC_DIR)/logic.s

memory.o: $(SRC_DIR)/memory.s
	$(CC) $(CFLAGS) -EL -c -x assembler-with-cpp -o $(BUILD_DIR)/memory.o $(SRC_DIR)/memory.s

clean:
	rm -f $(BUILD_DIR)/*.o $(BUILD_DIR)/*.elf $(BUILD_DIR)/*.rom
