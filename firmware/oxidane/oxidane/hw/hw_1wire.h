/**
 * \file
 * <!--
 * This file is part of BeRTOS.
 *
 * Bertos is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
 *
 *  Copyright (C) 2012 Robin Gilks
 * -->
 *
 * \addtogroup ow_driver 1-wire driver
 * \ingroup drivers
 * \{
 *
 *
 * \brief Driver for Dallas 1-wire devices
 *
 *
 * \author Peter Dannegger (danni(at)specs.de)
 * \author Martin Thomas (mthomas(at)rhrk.uni-kl.de)
 * \author Robin Gilks <g8ecj@gilks.org>
 *
 * $WIZ$ module_name = "hw_1wire"
 */

#ifndef HW_1WIRE_H
#define HW_1WIRE_H

#include "cfg/cfg_arch.h"
#include <cfg/compiler.h>
#include <cfg/macros.h>

#include <cpu/types.h>
#include <io/stm32.h>

#include <drv/gpio_stm32.h>
#include <drv/clock_stm32.h>
#include <drv/timer.h>

/**
 * \defgroup 1wirehw_api Hardware API
 * Access to this low level driver is mostly from the device specific layer. However,
 * some functions - especially the ow_set_bus() function operates at the lowest level.
 *
 * This functionality is especially useful when devices are hardwired and so removes
 * the need to scan them for their addresses.
 *
 * API usage example:
 * \code
 * switch (sensor)
 * {
 * case SENSOR_LOW:
 *    // low level sensor (ground) on PE4
 *    ow_set_bus (&PINE, &PORTE, &DDRE, PE4);
 *    if (!ow_busy ())                 // see if the conversion is complete
 *    {
 *       ow_ds18X20_read_temperature (NULL, &temperature_low);       // read the result
 *       ow_ds18X20_start (NULL, false]);            // start the conversion process again
 *    }
 *    break;
 * case SENSOR_HIGH:
 *    // high level (roof) sensor on PE5
 *    ow_set_bus (&PINE, &PORTE, &DDRE, PE5);
 *    if (!ow_busy ())                 // see if the conversion is complete
 *    {
 *       ow_ds18X20_read_temperature (NULL, &temperature_hi);       // read the result
 *       ow_ds18X20_start (NULL, false);            // start the conversion process again
 *    }
 *    break;
 * \endcode
 * \{
 */

#define OW_PIN   BV(4) //PB4

#define GPIO_BASE       ((struct stm32_gpio *)GPIOB_BASE)

#define OW_HW_PIN_ACTIVE()  \
	do { \
		stm32_gpioPinConfig(GPIO_BASE, OW_PIN, GPIO_MODE_OUT_PP, GPIO_SPEED_50MHZ); \
		stm32_gpioPinWrite(GPIO_BASE, OW_PIN, 0); \
	} while(0)

#define OW_HW_PIN_INACTIVE()  \
	do { \
		stm32_gpioPinConfig(GPIO_BASE, OW_PIN, GPIO_MODE_OUT_PP, GPIO_SPEED_50MHZ); \
		stm32_gpioPinWrite(GPIO_BASE, OW_PIN, 1); \
	} while(0)


INLINE bool ow_hw_pin_status(void)
{
	stm32_gpioPinConfig(GPIO_BASE, OW_PIN, GPIO_MODE_IN_FLOATING, GPIO_SPEED_50MHZ);
	return (bool)stm32_gpioPinRead(GPIO_BASE, OW_PIN);
}

/**
 * Enable parasitic mode (set line high to power device)
 */
#define OW_HW_PARASITE_ENABLE()
#define OW_HW_PARASITE_DISABLE()


/**
 * Init One Wire pin port
 */
#define OW_HW_INIT() \
	do { \
		/* Enable clocking on GPIOB */ \
		((struct RCC *)RCC_BASE)->AHBENR |= RCC_AHBENR_GPIOBEN; \
		stm32_gpioPinConfig(GPIO_BASE, OW_PIN, GPIO_MODE_OUT_PP, GPIO_SPEED_50MHZ); \
		stm32_gpioPinWrite(GPIO_BASE, CS, 1); \
	} while(0)

/** \} */ //defgroup 1wirehw_api

/** \} */ //addtogroup ow_driver

#endif

