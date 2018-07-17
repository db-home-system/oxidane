#include "si4455.h"

#include <string.h>

#ifdef VARIABLE_LENGTH_ON
    #warning Using variable length packets
    #include "configs/radio_config_vl_crc_pre10_sync4_pay8.h"
#else
    //#include "configs/radio_config_fixed_crc_pre10_sync4_pay8.h"
    #include "configs/custom.h"
#endif

#define SI4455_FIFO_SIZE 64
#define RADIO_CTS_TIMEOUT 1000

static const uint8_t DefaultRadioConfigurationDataArray[] = RADIO_CONFIGURATION_DATA_ARRAY;

static void powerUp(struct Si4455 *s);
static void reset(struct Si4455 *s);

static enum CommandResult initialize(struct Si4455 *s, const uint8_t* configArray);
static struct Si4455_IntStatus *readInterruptStatus(struct Si4455 *s, uint8_t clearPendingPH,
    uint8_t clearPendingModem, uint8_t clearPendingChip);

static struct Si4455_FrrA *readFrrA(struct Si4455 *s, uint8_t count);

static void writeEZConfigArray(struct Si4455 *s, const uint8_t* ezConfigArray, uint8_t count);
static void startTx(struct Si4455 *s, uint8_t condition);
static void startRx(struct Si4455 *s, uint8_t condition,
    uint8_t nextState1, uint8_t nextState2, uint8_t nextState3);
static void writeTxFifo(struct Si4455 *s, const uint8_t* data);

static bool processPHInterruptPending(struct Si4455 *s, uint8_t phPend);
static bool processModemInterruptPending(struct Si4455 *s, uint8_t modemPend);
static bool processChipInterruptPending(struct Si4455 *s, uint8_t chipPend);
static void setSystemError(struct Si4455 *s);
static void resetFifo(struct Si4455 *s);

static uint8_t getResponse(struct Si4455 *s, uint8_t* data, uint8_t count);
static void sendCommand(struct Si4455 *s, const uint8_t* data, uint8_t count);
static uint8_t sendCommandAndGetResponse(struct Si4455 *s, const uint8_t* commandData,
    uint8_t commandByteCount, uint8_t* responseData, uint8_t responseByteCount);

static void readData(struct Si4455 *s, uint8_t command, uint8_t* data, uint8_t count, bool pollCtsFlag);
static void writeData(struct Si4455 *s, uint8_t command, const uint8_t* data,
    uint8_t count, bool pollCtsFlag);
static uint8_t pollCts(struct Si4455 *s);
static void clearCts(struct Si4455 *s);

static uint8_t spiReadWriteByte(struct Si4455 *s, uint8_t value);
static void spiWriteByte(struct Si4455 *s, uint8_t value);
static uint8_t spiReadByte(struct Si4455 *s);
static void spiWriteData(struct Si4455 *s, const uint8_t* data, uint8_t count);
static void spiReadData(struct Si4455 *s, uint8_t* data, uint8_t count);


/*!
 * Initializes the radio module with the correct configuration.
 */
bool si4455_init(struct Si4455 *s, struct Si4455_Operations *ops)
{
    s->m_ops = ops;
    s->m_channelNumber = RADIO_CONFIGURATION_DATA_CHANNEL_NUMBER;
    s->m_packetLength = RADIO_CONFIGURATION_DATA_RADIO_PACKET_LENGTH;
    s->m_ctsWentHigh = 0;
    s->m_dataTransmittedFlag = false;
    s->m_dataAvailableFlag = false;
    s->m_crcErrorFlag = false;
    s->m_txFifoAlmostEmptyFlag = false;
    s->m_rxFifoAlmostFullFlag = false;
    s->m_systemError = false;

    // Power Up the radio chip
    powerUp(s);

    int retryCount = 10;

    // Load radio configuration
    while (initialize(s, DefaultRadioConfigurationDataArray) != Success && (retryCount--)) {
        // Wait and retry
        powerUp(s);
    }

    if (retryCount <= 0) {
        return false;
    }

    // Read ITs, clear pending ones
    readInterruptStatus(s, 0, 0, 0);

    return true;
}

/*!
 * Returns the current operating state of the device.
 */
enum DeviceState si4455_deviceState(struct Si4455 *s)
{
    return (enum DeviceState)(readFrrA(s, 1)->frr_a & 0x0F);
}

/*!
 * Send data to @a channel.
 *
 * @param data Pointer to data to send. When using variable length packets, first byte should be the payload length.
 * @param length Data length. Includes payload length byte when using variable length packets.
 */
void si4455_sendPacket(struct Si4455 *s, const uint8_t *data)
{
    if (!data || s->m_packetLength == 0) return;

    // Read ITs, clear pending ones
    readInterruptStatus(s, 0, 0, 0);

    if (s->m_systemError) return;

    // Wait when not ready
    enum DeviceState st = si4455_deviceState(s);
    uint16_t counter = 0xF000;  // Avoid stalling
    do {
        --counter;
        st = si4455_deviceState(s);
    } while ((st == Tx || st == TxTune) &&
             counter != 0);

    // TODO: Abort & set error flag?
    //if (counter == 0) // Abort, module not ready
    //    return;

    // Fill the TX fifo with data
    writeTxFifo(s, data);

    // Start sending packet on channel, return to RX after transmit
    startTx(s, 0x80);
}


/*!
 * Set Radio to RX mode, fixed packet length.
 *
 * @param channel Channel to listen to.
 * @param length Length of data to receive.
 */
void si4455_startListening(struct Si4455 *s)
{
    // Read ITs, clear pending ones
    readInterruptStatus(s, 0, 0, 0);

    // Start Receiving packet on channel, START immediately, Packet n bytes long
    startRx(s, 0,
            SI4455_CMD_START_RX_ARG_RXTIMEOUT_STATE_ENUM_RX,
            SI4455_CMD_START_RX_ARG_RXVALID_STATE_ENUM_RX,
            SI4455_CMD_START_RX_ARG_RXINVALID_STATE_ENUM_RX);
}

/*!
 * Returns true if a system error occured (auto clears).
 */
bool si4455_systemError(struct Si4455 *s)
{
    bool error = s->m_systemError;
    //m_systemError = false;
    return error;
}

/*!
 *  Power up the chip.
 */
static void powerUp(struct Si4455 *s)
{
  // Hardware reset the chip
  reset(s);

  // Wait until reset timeout or Reset IT signal
  //for (unsigned int wDelay = 0; wDelay < RadioConfiguration.Radio_Delay_Cnt_After_Reset; wDelay++);
  s->m_ops->delay(100);
}

/*!
 * Hardware reset the chip using shutdown input
 */
static void reset(struct Si4455 *s)
{
    // Put radio in shutdown, wait then release
    s->m_ops->assert_sdn();
    s->m_ops->delay(10);
    s->m_ops->deassert_sdn();
    s->m_ops->delay(10);
    clearCts(s);
}

/*!
 * Load all properties and commands with a list of NULL terminated commands.
 * Call @reset before.
 */
static enum CommandResult initialize(struct Si4455 *s, const uint8_t* configArray)
{
    // While cycle as far as the pointer points to a command
    while (*configArray != 0x00) {
        // Commands structure in the array:
        // --------------------------------
        // LEN | <LEN length of data>

        uint8_t cmdBytesCount = *configArray++;

        if (cmdBytesCount > 16u) {
            // Initial configuration of Si4x55
            if (*configArray == SI4455_CMD_ID_WRITE_TX_FIFO) {
                if (cmdBytesCount > 128u) {
                    // Number of command bytes exceeds maximal allowable length
                    // @todo May need to send NOP to send more than 128 bytes (check documentation)
                    return CommandError;
                }

                // Load array to the device
                configArray++;
                writeEZConfigArray(s, configArray, cmdBytesCount - 1);

                // Point to the next command
                configArray += cmdBytesCount - 1;

                // Continue command interpreter
                continue;
            } else {
                // Number of command bytes exceeds maximal allowable length
                return CommandError;
            }
        }

        uint8_t radioCmd[16];
        for (uint8_t col = 0; col < cmdBytesCount; col++) {
            radioCmd[col] = *configArray;
            configArray++;
        }

        uint8_t response = 0;
        if (sendCommandAndGetResponse(s, radioCmd, cmdBytesCount, &response, 1) != 0xFF) {
            // Timeout occured
            return CtsTimeout;
        }

        // Check response byte of EZCONFIG_CHECK command
        if (radioCmd[0] == SI4455_CMD_ID_EZCONFIG_CHECK) {
            if (response) {
                // Number of command bytes exceeds maximal allowable length
                return CommandError;
            }
        }

        if (s->m_ops->irq_asserted()) {
            // Get and clear all interrupts.  An error has occured...
            struct Si4455_IntStatus *it = readInterruptStatus(s, 0, 0, 0);

            if (it->chip_pend & SI4455_CMD_GET_CHIP_STATUS_REP_CMD_ERROR_PEND_MASK) {
                return CommandError;
            }
        }
    }

    return Success;
}

/*!
 * Writes data byte(s) to the EZConfig array (array generated from EZConfig tool).
 */
static void writeEZConfigArray(struct Si4455 *s, const uint8_t* ezConfigArray, uint8_t count)
{
    writeData(s, SI4455_CMD_ID_WRITE_TX_FIFO, ezConfigArray, count, true);
}

/*!
 * Switches to TX state and starts transmission of a packet.
 */
static void startTx(struct Si4455 *s, uint8_t condition)
{
    const uint8_t buffer[] = {
        SI4455_CMD_ID_START_TX,
        s->m_channelNumber,
        condition,
        (uint8_t)(s->m_packetLength >> 8),
        (uint8_t)(s->m_packetLength),
        0
    };

    sendCommand(s, buffer, SI4455_CMD_ARG_COUNT_START_TX);
}

/*!
 * Writes data byte(s) to the TX FIFO.
 */
static void writeTxFifo(struct Si4455 *s, const uint8_t* data)
{
    writeData(s, SI4455_CMD_ID_WRITE_TX_FIFO, data, s->m_packetLength, false);
}

/*!
 * Switches to RX state and starts reception of a packet.
 */
static void startRx(struct Si4455 *s, uint8_t condition,
    uint8_t nextState1, uint8_t nextState2, uint8_t nextState3)
{
    const uint8_t buffer[] = {
        SI4455_CMD_ID_START_RX,
        s->m_channelNumber,
        condition,
        (uint8_t)(s->m_packetLength >> 8),
        (uint8_t)(s->m_packetLength),
        nextState1,
        nextState2,
        nextState3
    };

    sendCommand(s, buffer, SI4455_CMD_ARG_COUNT_START_RX);
}

/*!
 * Returns the interrupt status of ALL the possible interrupt events (both STATUS and PENDING).
 * Optionally, it may be used to clear latched (PENDING) interrupt events.
 */
static struct Si4455_IntStatus *readInterruptStatus(struct Si4455 *s, uint8_t clearPendingPH,
    uint8_t clearPendingModem, uint8_t clearPendingChip)
{
    const uint8_t buffer[] = {
        SI4455_CMD_ID_GET_INT_STATUS,
        clearPendingPH,
        clearPendingModem,
        clearPendingChip
    };

    sendCommandAndGetResponse(s, buffer, SI4455_CMD_ARG_COUNT_GET_INT_STATUS,
                              s->m_commandReply.raw, SI4455_CMD_REPLY_COUNT_GET_INT_STATUS);

    if (s->m_systemError)
        return &s->m_commandReply.int_status; // TODO: Invalid data returned! Clear it before?


    processPHInterruptPending(s, s->m_commandReply.int_status.ph_pend);
    processModemInterruptPending(s, s->m_commandReply.int_status.modem_pend);
    processChipInterruptPending(s, s->m_commandReply.int_status.chip_pend);

    if (s->m_commandError) {
        s->m_commandError = false;
        si4455_startListening(s);
    }
    if (s->m_crcErrorFlag) {
        s->m_crcErrorFlag = false;
        si4455_startListening(s);
    }

    return &s->m_commandReply.int_status;
}

/*!
 * Process Packet Handler interrupts
 */
static bool processPHInterruptPending(struct Si4455 *s, uint8_t phPend)
{
    bool clearIT = false;

    if (phPend & SI4455_CMD_GET_INT_STATUS_REP_PACKET_SENT_PEND_BIT) {
        s->m_dataTransmittedFlag = true;
        clearIT = true;
    }
    if (phPend & SI4455_CMD_GET_INT_STATUS_REP_PACKET_RX_PEND_BIT) {
        // @todo Add circular buffer?
        s->m_dataAvailableFlag = true;
        clearIT = true;
    }

    if (phPend & SI4455_CMD_GET_INT_STATUS_REP_CRC_ERROR_PEND_BIT) {
        s->m_crcErrorFlag = true;
        resetFifo(s);
        clearIT = true;
    }

    if (phPend & SI4455_CMD_GET_INT_STATUS_REP_TX_FIFO_ALMOST_EMPTY_PEND_BIT) {
        s->m_txFifoAlmostEmptyFlag = true;
        clearIT = true;
    }

    if (phPend & SI4455_CMD_GET_INT_STATUS_REP_RX_FIFO_ALMOST_FULL_PEND_BIT) {
        s->m_rxFifoAlmostFullFlag = true;
        clearIT = true;
    }

    return clearIT;
}

/*!
 * Process Modem interrupts
 */
static bool processModemInterruptPending(struct Si4455 *s, uint8_t modemPend)
{
    bool clearIT = false;

    if (modemPend & SI4455_CMD_GET_INT_STATUS_REP_INVALID_SYNC_PEND_BIT) {
        clearIT = true;
    }

    if (modemPend & SI4455_CMD_GET_INT_STATUS_REP_INVALID_PREAMBLE_PEND_BIT) {
        clearIT = true;
    }

    if (modemPend & SI4455_CMD_GET_INT_STATUS_REP_PREAMBLE_DETECT_PEND_BIT) {
        //clearIT = true;
    }

    if (modemPend & SI4455_CMD_GET_INT_STATUS_REP_SYNC_DETECT_PEND_BIT) {
        //clearIT = true;
    }

    if (modemPend & SI4455_CMD_GET_INT_STATUS_REP_RSSI_PEND_BIT) {
        //clearIT = true;
    }

    return clearIT;
}

/*!
 * Process Chip interrupts
 */
static bool processChipInterruptPending(struct Si4455 *s, uint8_t chipPend)
{
    bool clearIT = false;

    if (chipPend & SI4455_CMD_GET_INT_STATUS_REP_FIFO_UNDERFLOW_OVERFLOW_ERROR_PEND_BIT) {
        resetFifo(s);
        clearIT = true;
    }

    if (chipPend & SI4455_CMD_GET_INT_STATUS_REP_CMD_ERROR_PEND_BIT) {
        resetFifo(s);
        s->m_commandError = true;
        clearIT = true;
    }

    if (chipPend & SI4455_CMD_GET_INT_STATUS_REP_STATE_CHANGE_PEND_BIT) {
        //clearIT = true;
    }
    if (chipPend & SI4455_CMD_GET_INT_STATUS_REP_CHIP_READY_PEND_BIT) {
        //clearIT = true;
    }

    return clearIT;
}

static void setSystemError(struct Si4455 *s)
{
    s->m_systemError = true;
}

/*!
 * Reset the internal FIFOs.
 */
static void resetFifo(struct Si4455 *s)
{
    const uint8_t buffer[] = {
        SI4455_CMD_ID_FIFO_INFO,
        0x03
    };

    sendCommand(s, buffer, SI4455_CMD_ARG_COUNT_FIFO_INFO);
}

/*!
 * Reports basic information about the device.
 */
struct Si4455_PartInfo *si4455_readPartInfo(struct Si4455 *s)
{
    const uint8_t buffer[] = {
        SI4455_CMD_ID_PART_INFO
    };

    // TODO: check PART value, seems like MSB is null and it shouldn't be.
    sendCommandAndGetResponse(s, buffer, SI4455_CMD_ARG_COUNT_PART_INFO,
                              s->m_commandReply.raw, SI4455_CMD_REPLY_COUNT_PART_INFO);

    return &s->m_commandReply.part_info;
}

/*!
 * Returns the Function revision information of the device.
 */
struct Si4455_FuncInfo *si4455_readFuncInfo(struct Si4455 *s)
{
    const uint8_t buffer[] = {
        SI4455_CMD_ID_FUNC_INFO
    };

    sendCommandAndGetResponse(s, buffer, SI4455_CMD_ARG_COUNT_FUNC_INFO,
                              s->m_commandReply.raw, SI4455_CMD_REPLY_COUNT_FUNC_INFO);

    return &s->m_commandReply.func_info;
}

/*!
 * Reads the fast response registers (FRR) starting with FRR_A.
 */
static struct Si4455_FrrA *readFrrA(struct Si4455 *s, uint8_t count)
{
    readData(s, SI4455_CMD_ID_FRR_A_READ,
             s->m_commandReply.raw,
             count,
             false);

    return &s->m_commandReply.frr_a;
}

/*!
 * Gets a command response from the radio chip.
 *
 * @param data  Pointer to where to put the data.
 * @param count Number of bytes to get from the radio chip.
 *
 * @return CTS value.
 */
static uint8_t getResponse(struct Si4455 *s, uint8_t* data, uint8_t count)
{
    uint8_t ctsVal = 0;
    uint16_t errorCount = RADIO_CTS_TIMEOUT;

    while (errorCount != 0) {   // Wait until radio IC is ready with the data
        s->m_ops->assert_cs();
        spiWriteByte(s, 0x44);     // Read CMD buffer
        ctsVal = spiReadByte(s);

        if (ctsVal == 0xFF) {
            if (count) {
                spiReadData(s, data, count);
            }
            s->m_ops->deassert_cs();
            break;
        }

        s->m_ops->deassert_cs();
        s->m_ops->delay(1);

        errorCount--;
    }

    if (errorCount == 0) {
        // ERROR! Should never take this long
        // @todo Error callback ?
        setSystemError(s);
        s->m_systemError = true; // Fix a strange "system error" bug...
        return 0;
    }

    if (ctsVal == 0xFF) {
        s->m_ctsWentHigh = true;
    }

    s->m_systemError = false;
    return ctsVal;
}

/*!
 * Sends a command to the radio chip.
 *
 * @param data  Pointer to the command to send.
 * @param count Number of bytes in the command to send to the radio device.
 */
static void sendCommand(struct Si4455 *s, const uint8_t* data, uint8_t count)
{
    while (!s->m_ctsWentHigh) {
        pollCts(s);
        if (s->m_systemError) return;
    }

    s->m_ops->assert_cs();
    spiWriteData(s, data, count);
    s->m_ops->deassert_cs();

    clearCts(s);
}

/*!
 * Sends a command to the radio chip and gets a response.
 *
 * @param commandData       Pointer to the command data.
 * @param commandByteCount  Number of bytes in the command to send to the radio device.
 * @param responseData      Pointer to where to put the response data.
 * @param responseByteCount Number of bytes in the response to fetch.
 *
 * @return CTS value.
 */
static uint8_t sendCommandAndGetResponse(struct Si4455 *s, const uint8_t* commandData,
    uint8_t commandByteCount, uint8_t* responseData, uint8_t responseByteCount)
{
    sendCommand(s, commandData, commandByteCount);
    return getResponse(s, responseData, responseByteCount);
}


/*!
 * Gets a command response from the radio chip.
 *
 * @param command     Command ID.
 * @param data        Pointer to where to put the data.
 * @param count       Number of bytes to get from the radio chip.
 * @param pollCtsFlag Set to poll CTS.
 */
static void readData(struct Si4455 *s, uint8_t command, uint8_t* data, uint8_t count, bool pollCtsFlag)
{
    if (pollCtsFlag) {
        while (!s->m_ctsWentHigh) {
            pollCts(s);
            if (s->m_systemError) return;
        }
    }

    s->m_ops->assert_cs();
    spiWriteByte(s, command);
    spiReadData(s, data, count);
    s->m_ops->deassert_cs();

    clearCts(s);
}

/*!
 * Gets a command response from the radio chip.
 *
 * @param command     Command ID.
 * @param data        Pointer to where to put the data.
 * @param count       Number of bytes to get from the radio chip.
 * @param pollCtsFlag Set to poll CTS.
 */
static void writeData(struct Si4455 *s, uint8_t command, const uint8_t* data,
    uint8_t count, bool pollCtsFlag)
{
    if (pollCtsFlag) {
        while (!s->m_ctsWentHigh) {
            pollCts(s);
            if (s->m_systemError) return;
        }
    }

    s->m_ops->assert_cs();
    spiWriteByte(s, command);
    spiWriteData(s, data, count);
    s->m_ops->deassert_cs();

    clearCts(s);
}


/*!
 * Waits for CTS to be high.
 *
 * @return CTS value.
 */
static uint8_t pollCts(struct Si4455 *s)
{
    return getResponse(s, NULL, 0);
}

/*!
 * Clears the CTS state variable.
 */
static void clearCts(struct Si4455 *s)
{
    s->m_ctsWentHigh = false;
}

static uint8_t spiReadWriteByte(struct Si4455 *s, uint8_t value)
{
    uint8_t ret;
    s->m_ops->xfer(&value, &ret, 1);
    return ret;
}

static void spiWriteByte(struct Si4455 *s, uint8_t value)
{
    spiReadWriteByte(s, value);
}

static uint8_t spiReadByte(struct Si4455 *s)
{
    return spiReadWriteByte(s, 0xFF);
}

static void spiWriteData(struct Si4455 *s, const uint8_t* data, uint8_t count)
{
    s->m_ops->transmit(data, count);
}

static void spiReadData(struct Si4455 *s, uint8_t* data, uint8_t count)
{
    memset(data, 0xFF, count);
    s->m_ops->receive(data, count);
}
