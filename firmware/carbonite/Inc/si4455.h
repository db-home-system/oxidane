#ifndef __SI4455_H
#define __SI4455_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include "si4455_defs.h"

/* #define VARIABLE_LENGTH_ON */
/* #define SI4455_DEBUG_VERBOSE_ON */

enum DeviceState
{
	Sleep = 1,
	SpiActive = 2,
	Ready = 3,
	Ready2 = 4,
	TxTune = 5,
	RxTune = 6,
	Tx = 7,
	Rx = 8
};

enum CommandResult
{
	Success,
	NoPatch,
	CtsTimeout,
	PatchFail,
	CommandError
};

struct Si4455_Operations {
	/* Sleep for n milliseconds */
	void (*delay)(int n);
	/* Set SDN pin high */
	void (*assert_sdn)(void);
	/* Set SDN pin low */
	void (*deassert_sdn)(void);
	/* Put CS low for transfer */
	void (*assert_cs)(void);
	/* Put CS high */
	void (*deassert_cs)(void);
	/* Check if IRQ is low */
	bool (*irq_asserted)(void);
	/* Send count bytes from data on SPI */
	void (*transmit)(const uint8_t *data, uint8_t count);
	/* Receive count bytes in data on SPI */
	void (*receive)(uint8_t *data, uint8_t count);
	/* Transfers count bytes from tx and receives count bytes in rx on SPI */
	void (*xfer)(const uint8_t *tx, uint8_t *rx, uint8_t count);
};

struct Si4455 {
	struct Si4455_Operations *m_ops;

	uint8_t m_channelNumber;
	uint8_t m_packetLength;

	union si4455_cmd_reply_union m_commandReply;

	bool m_ctsWentHigh;
	bool m_dataTransmittedFlag;
	bool m_dataAvailableFlag;
	bool m_crcErrorFlag;
	bool m_txFifoAlmostEmptyFlag;
	bool m_rxFifoAlmostFullFlag;
	bool m_commandError;
	bool m_systemError;
};

bool si4455_init(struct Si4455 *s, struct Si4455_Operations *ops);

void si4455_sendPacket(struct Si4455 *s, const uint8_t *data);
void si4455_startListening(struct Si4455 *s);

enum DeviceState si4455_deviceState(struct Si4455 *s);
struct Si4455_PartInfo *si4455_readPartInfo(struct Si4455 *s);
struct Si4455_FuncInfo *si4455_readFuncInfo(struct Si4455 *s);

bool si4455_isAlive(struct Si4455 *s);

#endif /* __SI4455_H */
