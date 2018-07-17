#ifndef __RADIO_H__
#define __RADIO_H__

#include <stdbool.h>
#include "stm32l1xx_hal.h"

bool radio_init(SPI_HandleTypeDef *hspi);
bool radio_transmit(const uint8_t *data, uint16_t len);

#endif /* __RADIO_H__ */
