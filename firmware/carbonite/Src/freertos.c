#include "FreeRTOS.h"
#include "task.h"
#include "stm32l1xx_hal.h"

/* Pre/Post sleep processing prototypes */
void PreSleepProcessing(uint32_t *ulExpectedIdleTime);
void PostSleepProcessing(uint32_t *ulExpectedIdleTime);

/* GetIdleTaskMemory prototype (linked to static allocation support) */
void vApplicationGetIdleTaskMemory(StaticTask_t **ppxIdleTaskTCBBuffer,
	StackType_t **ppxIdleTaskStackBuffer, uint32_t *pulIdleTaskStackSize);

void PreSleepProcessing(uint32_t *ulExpectedIdleTime)
{
}

void PostSleepProcessing(uint32_t *ulExpectedIdleTime)
{
}

static StaticTask_t xIdleTaskTCBBuffer;
static StackType_t xIdleStack[configMINIMAL_STACK_SIZE];

void vApplicationGetIdleTaskMemory( StaticTask_t **ppxIdleTaskTCBBuffer, StackType_t **ppxIdleTaskStackBuffer, uint32_t *pulIdleTaskStackSize )
{
	*ppxIdleTaskTCBBuffer = &xIdleTaskTCBBuffer;
	*ppxIdleTaskStackBuffer = &xIdleStack[0];
	*pulIdleTaskStackSize = configMINIMAL_STACK_SIZE;
}
