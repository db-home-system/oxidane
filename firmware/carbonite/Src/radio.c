#include "radio.h"
#include "si4455.h"
#include "main.h"

#define SPI_TIMEOUT 1000

static SPI_HandleTypeDef *spi;

static void si4455_delay(int n)
{
    HAL_Delay(n);
}

static void si4455_assert_sdn(void)
{
    HAL_GPIO_WritePin(SI4455_SDN_GPIO_Port, SI4455_SDN_Pin, GPIO_PIN_SET);
}

static void si4455_deassert_sdn(void)
{
    HAL_GPIO_WritePin(SI4455_SDN_GPIO_Port, SI4455_SDN_Pin, GPIO_PIN_RESET);
}

static void si4455_assert_cs(void)
{
    HAL_GPIO_WritePin(SI4455_CS_GPIO_Port, SI4455_CS_Pin, GPIO_PIN_RESET);
}

static void si4455_deassert_cs(void)
{
    HAL_GPIO_WritePin(SI4455_CS_GPIO_Port, SI4455_CS_Pin, GPIO_PIN_SET);
}

static bool si4455_irq_asserted(void)
{
    return HAL_GPIO_ReadPin(SI4455_IRQ_GPIO_Port, SI4455_IRQ_Pin) == GPIO_PIN_RESET;
}

static void si4455_transmit(const uint8_t *data, uint8_t count)
{
    HAL_SPI_Transmit(spi, (uint8_t *) data, count, SPI_TIMEOUT);
}

static void si4455_receive(uint8_t *data, uint8_t count)
{
    HAL_SPI_Receive(spi, data, count, SPI_TIMEOUT);
}

static void si4455_xfer(const uint8_t *tx, uint8_t *rx, uint8_t count)
{
    HAL_SPI_TransmitReceive(spi, (uint8_t *) tx, rx, count, SPI_TIMEOUT);
}

static struct Si4455_Operations ops = {
    .delay = si4455_delay,
    .assert_sdn = si4455_assert_sdn,
    .deassert_sdn = si4455_deassert_sdn,
    .assert_cs = si4455_assert_cs,
    .deassert_cs = si4455_deassert_cs,
    .irq_asserted = si4455_irq_asserted,
    .transmit = si4455_transmit,
    .receive = si4455_receive,
    .xfer = si4455_xfer,
};

static struct Si4455 s;

bool radio_init(SPI_HandleTypeDef *hspi)
{
    spi = hspi;

    return si4455_init(&s, &ops);
}

bool radio_transmit(const uint8_t *data, uint16_t len)
{
    si4455_sendPacket(&s, data);
    return true;
}
