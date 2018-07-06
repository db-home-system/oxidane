#include "stm32l1xx_hal.h"

extern void _Error_Handler(char *, int);

/**
  * Initializes the Global MSP.
  */
void HAL_MspInit(void)
{
	__HAL_RCC_COMP_CLK_ENABLE();
	__HAL_RCC_SYSCFG_CLK_ENABLE();
	__HAL_RCC_PWR_CLK_ENABLE();

	HAL_NVIC_SetPriorityGrouping(NVIC_PRIORITYGROUP_4);

	/* System interrupt init*/
	HAL_NVIC_SetPriority(MemoryManagement_IRQn, 0, 0);
	HAL_NVIC_SetPriority(BusFault_IRQn, 0, 0);
	HAL_NVIC_SetPriority(UsageFault_IRQn, 0, 0);
	HAL_NVIC_SetPriority(SVC_IRQn, 0, 0);
	HAL_NVIC_SetPriority(DebugMonitor_IRQn, 0, 0);
	HAL_NVIC_SetPriority(PendSV_IRQn, 15, 0);
	HAL_NVIC_SetPriority(SysTick_IRQn, 15, 0);
}

void HAL_ADC_MspInit(ADC_HandleTypeDef* hadc)
{
	GPIO_InitTypeDef GPIO_InitStruct;

	if (hadc->Instance == ADC1)
	{
		/* Peripheral clock enable */
		__HAL_RCC_ADC1_CLK_ENABLE();

		GPIO_InitStruct.Pin = AN0_Pin | AN1_Pin | AN2_Pin | AN3_Pin;
		GPIO_InitStruct.Mode = GPIO_MODE_ANALOG;
		GPIO_InitStruct.Pull = GPIO_NOPULL;

		HAL_GPIO_Init(GPIOA, &GPIO_InitStruct);
	}
}

void HAL_ADC_MspDeInit(ADC_HandleTypeDef* hadc)
{
	if (hadc->Instance == ADC1)
	{
		/* Peripheral clock disable */
		__HAL_RCC_ADC1_CLK_DISABLE();

		HAL_GPIO_DeInit(GPIOA, AN0_Pin | AN1_Pin | AN2_Pin | AN3_Pin);
	}
}

void HAL_I2C_MspInit(I2C_HandleTypeDef* hi2c)
{
	GPIO_InitTypeDef GPIO_InitStruct;

	if (hi2c->Instance == I2C1)
	{
		/* Peripheral clock enable */
		__HAL_RCC_I2C1_CLK_ENABLE();

		GPIO_InitStruct.Pin = GPIO_PIN_8 | GPIO_PIN_9;
		GPIO_InitStruct.Mode = GPIO_MODE_AF_OD;
		GPIO_InitStruct.Pull = GPIO_PULLUP;
		GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_VERY_HIGH;
		GPIO_InitStruct.Alternate = GPIO_AF4_I2C1;

		HAL_GPIO_Init(GPIOB, &GPIO_InitStruct);
	}
}

void HAL_I2C_MspDeInit(I2C_HandleTypeDef* hi2c)
{
	if (hi2c->Instance == I2C1)
	{
		/* Peripheral clock disable */
		__HAL_RCC_I2C1_CLK_DISABLE();

		HAL_GPIO_DeInit(GPIOB, GPIO_PIN_8 | GPIO_PIN_9);
	}
}

void HAL_RTC_MspInit(RTC_HandleTypeDef* hrtc)
{
	if (hrtc->Instance == RTC)
	{
		/* Peripheral clock enable */
		__HAL_RCC_RTC_ENABLE();
	}
}

void HAL_RTC_MspDeInit(RTC_HandleTypeDef* hrtc)
{
	if (hrtc->Instance == RTC)
	{
		/* Peripheral clock disable */
		__HAL_RCC_RTC_DISABLE();
	}
}

void HAL_SPI_MspInit(SPI_HandleTypeDef* hspi)
{
	GPIO_InitTypeDef GPIO_InitStruct;

	if (hspi->Instance == SPI1)
	{
		/* Peripheral clock enable */
		__HAL_RCC_SPI1_CLK_ENABLE();

		GPIO_InitStruct.Pin = SI4455_SCLK_Pin | SI4455_SDO_Pin | SI4455_SDI_Pin;
		GPIO_InitStruct.Mode = GPIO_MODE_AF_PP;
		GPIO_InitStruct.Pull = GPIO_NOPULL;
		GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_VERY_HIGH;
		GPIO_InitStruct.Alternate = GPIO_AF5_SPI1;

		HAL_GPIO_Init(GPIOA, &GPIO_InitStruct);
	}
}

void HAL_SPI_MspDeInit(SPI_HandleTypeDef* hspi)
{
	if (hspi->Instance == SPI1)
	{
		/* Peripheral clock disable */
		__HAL_RCC_SPI1_CLK_DISABLE();

		HAL_GPIO_DeInit(GPIOA, SI4455_SCLK_Pin | SI4455_SDO_Pin | SI4455_SDI_Pin);
	}
}

void HAL_UART_MspInit(UART_HandleTypeDef* huart)
{
	GPIO_InitTypeDef GPIO_InitStruct;

	if (huart->Instance == USART1)
	{
		/* Peripheral clock enable */
		__HAL_RCC_USART1_CLK_ENABLE();

		GPIO_InitStruct.Pin = DBG_TX_Pin | DBG_RX_Pin;
		GPIO_InitStruct.Mode = GPIO_MODE_AF_PP;
		GPIO_InitStruct.Pull = GPIO_PULLUP;
		GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_VERY_HIGH;
		GPIO_InitStruct.Alternate = GPIO_AF7_USART1;

		HAL_GPIO_Init(GPIOA, &GPIO_InitStruct);
	}
}

void HAL_UART_MspDeInit(UART_HandleTypeDef* huart)
{
	if (huart->Instance == USART1)
	{
		/* Peripheral clock disable */
		__HAL_RCC_USART1_CLK_DISABLE();

		HAL_GPIO_DeInit(GPIOA, DBG_TX_Pin | DBG_RX_Pin);
	}
}

