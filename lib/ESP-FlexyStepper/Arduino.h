#ifndef _ARDUILNO_H_
#define _ARDUILNO_H_

// Это заглушка для ESP32, чтобы не использовать Arduino.h

#include <math.h>
#include <stdint.h>

using TaskHandle_t = void *;
using Task_t = void (*)(void *);
using byte = uint8_t;
using BaseType_t = uint32_t;
using ulong = uint32_t;

constexpr uint8_t OUTPUT = 0x03;
constexpr uint8_t PULLUP = 0x04;
constexpr uint8_t INPUT_PULLUP = 0x05;
constexpr uint8_t PULLDOWN = 0x08;
constexpr uint8_t INPUT_PULLDOWN = 0x09;
constexpr uint8_t OPEN_DRAIN = 0x10;
constexpr uint8_t OUTPUT_OPEN_DRAIN = 0x13;
constexpr uint8_t ANALOG = 0xC0;

constexpr uint8_t HIGH = 1;
constexpr uint8_t LOW = 0;

constexpr ulong LONG_MAX = 0xFFFFFFFF;

#ifndef NULL
#define NULL (decltype(nullptr))0
#endif

extern "C"
{
    void disableCore0WDT();
    void disableCore1WDT();

    void xTaskCreatePinnedToCore(Task_t task, const char *name, uint32_t stackSize, void *param, uint32_t priority, TaskHandle_t *taskHandle, BaseType_t coreID);
    void vTaskDelete(TaskHandle_t h);

    long uxTaskGetStackHighWaterMark(TaskHandle_t h);

    void configASSERT(TaskHandle_t h);

    void pinMode(uint8_t pin, uint8_t mode);

    void digitalWrite(uint8_t pin, uint8_t value);
    uint8_t digitalRead(uint8_t pin);

    void delay(uint32_t ms);

    ulong millis();
    ulong micros();
}

#endif