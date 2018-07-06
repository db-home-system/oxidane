#ifndef __MAIN_H__
#define __MAIN_H__

#define SI4455_IRQ_Pin GPIO_PIN_13
#define SI4455_IRQ_GPIO_Port GPIOC
#define AN0_Pin GPIO_PIN_0
#define AN0_GPIO_Port GPIOA
#define AN1_Pin GPIO_PIN_1
#define AN1_GPIO_Port GPIOA
#define AN2_Pin GPIO_PIN_2
#define AN2_GPIO_Port GPIOA
#define AN3_Pin GPIO_PIN_3
#define AN3_GPIO_Port GPIOA
#define SI4455_CS_Pin GPIO_PIN_4
#define SI4455_CS_GPIO_Port GPIOA
#define SI4455_SCLK_Pin GPIO_PIN_5
#define SI4455_SCLK_GPIO_Port GPIOA
#define SI4455_SDO_Pin GPIO_PIN_6
#define SI4455_SDO_GPIO_Port GPIOA
#define SI4455_SDI_Pin GPIO_PIN_7
#define SI4455_SDI_GPIO_Port GPIOA
#define BOARD_ID0_Pin GPIO_PIN_0
#define BOARD_ID0_GPIO_Port GPIOB
#define BOARD_ID1_Pin GPIO_PIN_1
#define BOARD_ID1_GPIO_Port GPIOB
#define SI4455_SDN_Pin GPIO_PIN_10
#define SI4455_SDN_GPIO_Port GPIOB
#define VEN_RF_Pin GPIO_PIN_11
#define VEN_RF_GPIO_Port GPIOB
#define VEN_I2C_Pin GPIO_PIN_12
#define VEN_I2C_GPIO_Port GPIOB
#define DBG_TX_Pin GPIO_PIN_9
#define DBG_TX_GPIO_Port GPIOA
#define DBG_RX_Pin GPIO_PIN_10
#define DBG_RX_GPIO_Port GPIOA
#define VEN_EXT_Pin GPIO_PIN_3
#define VEN_EXT_GPIO_Port GPIOB

/**
  * @brief Uncomment the line below to expanse the "assert_param" macro in the
  *        HAL drivers code
  */
/* #define USE_FULL_ASSERT    1U */

#ifdef __cplusplus
 extern "C" {
#endif

void _Error_Handler(char *, int);
#define Error_Handler() _Error_Handler(__FILE__, __LINE__)

#ifdef __cplusplus
}
#endif

#endif /* __MAIN_H__ */
