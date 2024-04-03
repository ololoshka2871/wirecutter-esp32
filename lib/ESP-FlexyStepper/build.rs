fn main() {
    println!("cargo:rustc-env=CROSS_COMPILE=xtensa-esp32-elf"); 
    std::env::set_var("CROSS_COMPILE", "xtensa-esp32-elf");

    let src = ["ESP-FlexyStepper-dist/src/ESP_FlexyStepper.cpp"];

    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include("ESP-FlexyStepper-dist/src")
        .include(".")
        .flag("-Wno-conversion-null")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-pointer-arith")
        .define("ESP32", Some("0"));
    build.compile("ESP-FlexyStepper");

    //rerun if changed any("ESP-FlexyStepper-dist/src/ESP_FlexyStepper.cpp");
    for f in src.iter() {
        println!("cargo:rerun-if-changed={}", f);
    }
    println!("cargo:rerun-if-changed=Arduino.h");
}
