# Rust workspace for Raspberry Pi 3 feL4/seL4 things

NOTE: most of these are a WIP and in an experimental/hacky state

Some of the projects/libs:

- [bcm2837/](bcm2837/): BCM2837 device crate
- [bcm2837-hal/](bcm2837-hal/): embedded-hal traits for the BCM2837
- [display/](display/): Double-buffered, DMA backed graphics display library
- [clock-display/](clock-display/): Example project, render a simple clock
- [dma-example/](dma-example/): Example project, DMA transfer to GPU memory
- [spi-enc28j60-eth-example/](spi-enc28j60-eth-example/): Example Ethernet driver

## Dependencies

* ~~[cargo-fel4](https://github.com/maindotrs/cargo-fel4)~~
* [cargo-fel4 with my rpi3 changes](https://github.com/jonlamb-gh/cargo-fel4/tree/add-rpi3-support-and-custom-root-task)

## U-boot

Using 64 bit U-boot:

```bash
U-Boot> version
U-Boot 2018.11-g208ecba (Nov 14 2018 - 13:17:50 -0800)

aarch64-linux-gnu-gcc (Linaro GCC 7.3-2018.05) 7.3.1 20180425 [linaro-7.3-2018.05 revision d29120a424ec
fbc167ef90065c0eeb7f91977701]
GNU ld (Linaro_Binutils-2018.05) 2.28.2.20170706
```

Environment:

```bash
setenv imgname feL4img
setenv loadaddr 0x20000000

setenv bootfeL4 'tftp ${loadaddr} ${serverip}:${imgname}; dcache flush; dcache off; go ${loadaddr}'
```

## SD Card

Files:

```bash
/card
├── bootcode.bin
├── config.txt
├── fixup.dat
├── start.elf
├── u-boot.bin
└── uboot.env
```

Contents of `config.txt`:

```bash
enable_uart=1
arm_64bit=1
dtoverlay=pi3-disable-bt
kernel=u-boot.bin
```

## Simulating

TODO - need to add custom simulation configs to cargo-fel4 so it's easier to
enble graphics/etc

By default, the underlying seL4 `simulate` script supplies QEMU with `--no-graphic`.

If you want to enable graphics (to see the GPU/framebuffer examples), add the extra arg `--graphic=`.
