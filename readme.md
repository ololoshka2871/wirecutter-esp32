# WireCutter-ESP32

## Flash
Необходима утилита `espflash`
```shell
cargo install espflash@1.5.0
```

1. Вывезти ESP32 в режим программирования BOOT0 + Reset
2. Выполнить
```shell
espflash <SERIAL_PORT> target/xtensa-esp32-espidf/{debug,release}/wirecutter
```