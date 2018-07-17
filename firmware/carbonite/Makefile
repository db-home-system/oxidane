BUILD_DIR ?= build
DEBUG ?= 0

TARGET = carbonite
OPT = -Os
PREFIX = arm-none-eabi-

ifdef GCC_PATH
CC = $(GCC_PATH)/$(PREFIX)gcc
AS = $(GCC_PATH)/$(PREFIX)gcc -x assembler-with-cpp
CP = $(GCC_PATH)/$(PREFIX)objcopy
SZ = $(GCC_PATH)/$(PREFIX)size
else
CC = $(PREFIX)gcc
AS = $(PREFIX)gcc -x assembler-with-cpp
CP = $(PREFIX)objcopy
SZ = $(PREFIX)size
endif

HEX = $(CP) -O ihex
BIN = $(CP) -O binary -S

C_SOURCES =  \
	Src/main.c \
	Src/freertos.c \
	Src/si4455.c\
	Src/stm32l1xx_it.c \
	Src/stm32l1xx_hal_msp.c \
	Src/stm32l1xx_hal_timebase_TIM.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_adc.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_adc_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_i2c.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_rtc.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_rtc_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_spi.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_spi_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_tim.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_tim_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_uart.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_rcc.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_rcc_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_flash.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_flash_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_flash_ramfunc.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_gpio.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_dma.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_pwr.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_pwr_ex.c \
	Drivers/STM32L1xx_HAL_Driver/Src/stm32l1xx_hal_cortex.c \
	Src/system_stm32l1xx.c \
	Middlewares/Third_Party/FreeRTOS/Source/portable/GCC/ARM_CM3/port.c \
	Middlewares/Third_Party/FreeRTOS/Source/list.c \
	Middlewares/Third_Party/FreeRTOS/Source/queue.c \
	Middlewares/Third_Party/FreeRTOS/Source/timers.c \
	Middlewares/Third_Party/FreeRTOS/Source/tasks.c \
	Middlewares/Third_Party/FreeRTOS/Source/event_groups.c \
	Middlewares/Third_Party/FreeRTOS/Source/croutine.c \
	Middlewares/Third_Party/FreeRTOS/Source/CMSIS_RTOS/cmsis_os.c

ASM_SOURCES =  \
	startup_stm32l151xb.s

C_INCLUDES =  \
	-IInc \
	-IDrivers/STM32L1xx_HAL_Driver/Inc \
	-IDrivers/STM32L1xx_HAL_Driver/Inc/Legacy \
	-IMiddlewares/Third_Party/FreeRTOS/Source/portable/GCC/ARM_CM3 \
	-IDrivers/CMSIS/Device/ST/STM32L1xx/Include \
	-IMiddlewares/Third_Party/FreeRTOS/Source/include \
	-IMiddlewares/Third_Party/FreeRTOS/Source/CMSIS_RTOS \
	-IDrivers/CMSIS/Include

AS_INCLUDES =  \
	-I/Inc

AS_DEFS =
C_DEFS = -DUSE_HAL_DRIVER -DSTM32L151xB

CPU = -mcpu=cortex-m3
MCU = $(CPU) -mthumb $(FPU) $(FLOAT-ABI)
ASFLAGS = $(MCU) $(AS_DEFS) $(AS_INCLUDES) $(OPT) -Wall -fdata-sections -ffunction-sections
CFLAGS = $(MCU) $(C_DEFS) $(C_INCLUDES) $(OPT) -Wall -fdata-sections -ffunction-sections

ifeq ($(DEBUG), 1)
CFLAGS += -g -gdwarf-2
endif

CFLAGS += -MMD -MP -MF"$(@:%.o=%.d)"


LDSCRIPT = STM32L151C8Ux_FLASH.ld

LIBS = -lc -lm -lnosys
LIBDIR =
LDFLAGS = $(MCU) -specs=nano.specs -T$(LDSCRIPT) $(LIBDIR) $(LIBS) \
	-Wl,-Map=$(BUILD_DIR)/$(TARGET).map,--cref -Wl,--gc-sections

OBJECTS = $(addprefix $(BUILD_DIR)/,$(notdir $(C_SOURCES:.c=.o)))
vpath %.c $(sort $(dir $(C_SOURCES)))
OBJECTS += $(addprefix $(BUILD_DIR)/,$(notdir $(ASM_SOURCES:.s=.o)))
vpath %.s $(sort $(dir $(ASM_SOURCES)))

all: $(BUILD_DIR)/$(TARGET).elf $(BUILD_DIR)/$(TARGET).hex $(BUILD_DIR)/$(TARGET).bin

$(BUILD_DIR)/%.o: %.c Makefile | $(BUILD_DIR)
	$(CC) -c $(CFLAGS) -Wa,-a,-ad,-alms=$(BUILD_DIR)/$(notdir $(<:.c=.lst)) $< -o $@

$(BUILD_DIR)/%.o: %.s Makefile | $(BUILD_DIR)
	$(AS) -c $(CFLAGS) $< -o $@

$(BUILD_DIR)/$(TARGET).elf: $(OBJECTS) Makefile
	$(CC) $(OBJECTS) $(LDFLAGS) -o $@
	$(SZ) $@

$(BUILD_DIR)/%.hex: $(BUILD_DIR)/%.elf | $(BUILD_DIR)
	$(HEX) $< $@

$(BUILD_DIR)/%.bin: $(BUILD_DIR)/%.elf | $(BUILD_DIR)
	$(BIN) $< $@

$(BUILD_DIR):
	mkdir $@

clean:
	-rm -fR $(BUILD_DIR)

-include $(wildcard $(BUILD_DIR)/*.d)
