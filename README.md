# Rust workspace for Raspberry Pi 3 feL4/seL4 things

## Dependencies

* [cargo-fel4](https://github.com/maindotrs/cargo-fel4)

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
