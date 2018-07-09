#ifndef __STM32L1xx_IT_H
#define __STM32L1xx_IT_H

#ifdef __cplusplus
extern "C" {
#endif

#include "stm32l1xx_hal.h"
#include "main.h"

void NMI_Handler(void);
void HardFault_Handler(void);
void MemManage_Handler(void);
void BusFault_Handler(void);
void UsageFault_Handler(void);
void DebugMon_Handler(void);
void SysTick_Handler(void);
void TIM6_IRQHandler(void);

#ifdef __cplusplus
}
#endif

#endif /* __STM32L1xx_IT_H */
