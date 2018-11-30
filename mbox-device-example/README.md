# TODO


## Dependencies

TODO

```bash
cargo install cargo-fel4
```

## Building

```bash
cargo fel4 build
```

## Simulating

```bash
cargo fel4 simulate
```

```text
qemu-system-aarch64 -machine raspi3  -nographic -s -serial null -serial mon:stdio -m size=1024M  -kernel debug/kernel -initrd debug/feL4img

ELF-loader started on CPU: ARM Ltd. Cortex-A53 r0p4
  paddr=[80000..1f7fff]
ELF-loading image 'kernel'
  paddr=[1000000..122ffff]
  vaddr=[ffffff8000000000..ffffff800022ffff]
  virt_entry=ffffff8000000000
ELF-loading image 'root-task'
  paddr=[1230000..1260fff]
  vaddr=[400000..430fff]
  virt_entry=4004b8
Enabling MMU and paging
Jumping to kernel-image entry point...

Bootstrapping kernel
Booting all finished, dropped to user space
------------- bootinfo -------------
bootinfo.empty.start = 131
bootinfo.empty.end = 524288
bootinfo.userImageFrames.start = 15
bootinfo.userImageFrames.end = 96
bootinfo.untyped.start = 96
bootinfo.untyped.end = 131
bootinfo.untypedList
  length = 35
  [0 | 96] paddr = 0x1000000 - size_bits = 16 - is_device = 0
  [1 | 97] paddr = 0x1282000 - size_bits = 13 - is_device = 0
  [2 | 98] paddr = 0x1284000 - size_bits = 14 - is_device = 0
  [3 | 99] paddr = 0x1288000 - size_bits = 15 - is_device = 0
  [4 | 100] paddr = 0x1290000 - size_bits = 16 - is_device = 0
  [5 | 101] paddr = 0x12A0000 - size_bits = 17 - is_device = 0
  [6 | 102] paddr = 0x12C0000 - size_bits = 18 - is_device = 0
  [7 | 103] paddr = 0x1300000 - size_bits = 20 - is_device = 0
  [8 | 104] paddr = 0x1400000 - size_bits = 22 - is_device = 0
  [9 | 105] paddr = 0x1800000 - size_bits = 23 - is_device = 0
  [10 | 106] paddr = 0x2000000 - size_bits = 24 - is_device = 0
  [11 | 107] paddr = 0x3000000 - size_bits = 25 - is_device = 0
  [12 | 108] paddr = 0x5000000 - size_bits = 24 - is_device = 0
  [13 | 109] paddr = 0x6000000 - size_bits = 23 - is_device = 0
  [14 | 110] paddr = 0x6800000 - size_bits = 22 - is_device = 0
  [15 | 111] paddr = 0x6C00000 - size_bits = 21 - is_device = 0
  [16 | 112] paddr = 0x6E00000 - size_bits = 20 - is_device = 0
  [17 | 113] paddr = 0x6F00000 - size_bits = 19 - is_device = 0
  [18 | 114] paddr = 0x6F80000 - size_bits = 18 - is_device = 0
  [19 | 115] paddr = 0x6FC0000 - size_bits = 17 - is_device = 0
  [20 | 116] paddr = 0x6FE0000 - size_bits = 16 - is_device = 0
  [21 | 117] paddr = 0x6FF0000 - size_bits = 14 - is_device = 0
  [22 | 118] paddr = 0x6FF4000 - size_bits = 13 - is_device = 0
  [23 | 119] paddr = 0x3C000000 - size_bits = 24 - is_device = 1
  [24 | 120] paddr = 0x3D000000 - size_bits = 25 - is_device = 1
  [25 | 121] paddr = 0x3F00B000 - size_bits = 12 - is_device = 1
  [26 | 122] paddr = 0x3F100000 - size_bits = 12 - is_device = 1
  [27 | 123] paddr = 0x3F104000 - size_bits = 12 - is_device = 1
  [28 | 124] paddr = 0x3F200000 - size_bits = 12 - is_device = 1
  [29 | 125] paddr = 0x3F201000 - size_bits = 12 - is_device = 1
  [30 | 126] paddr = 0x3F300000 - size_bits = 12 - is_device = 1
  [31 | 127] paddr = 0x3F980000 - size_bits = 12 - is_device = 1
  [32 | 128] paddr = 0x3F215000 - size_bits = 12 - is_device = 1
  [33 | 129] paddr = 0x3F00B000 - size_bits = 12 - is_device = 1
  [34 | 130] paddr = 0x3F007000 - size_bits = 12 - is_device = 1
--------------------------

Hello from custom init fn

Mapped VideoCore Mailbox device region
  vaddr = 0x10000880 paddr = 0x3F00B880
  base vaddr = 0x10000000 base_paddr 0x3F00B000
Mapped GPIO device region
  vaddr = 0x10001000 paddr = 0x3F200000
Mapped UART1 device region
  vaddr = 0x10002000 paddr = 0x3F215000

This is output from a Serial<UART1>

Allocated pmem page
  vaddr = 0x10003000 paddr = 0x1264000

Mailbox send GetSerialNumCmd

Response = GetSerialNumResp(
    GetSerialNumResp {
        serial_number: 3112636479
    }
)

Mailbox send GetTemperatureCmd

Response = GetTemperatureResp(
    GetTemperatureResp {
        id: 0,
        value: 42932
    }
)

Mailbox send GetArmMemCmd

Response = GetArmMemResp(
    GetArmMemResp {
        addr: 0,
        size: 994050048 (0x3B40_0000)
    }
)

Mailbox send GetVcMemCmd

Response = GetVcMemResp(
    GetVcMemResp {
        addr: 994050048 (0x3B40_0000),
        size: 79691776 (0x4C0_0000)
        top: 0x4000_0000
    }
)

Mailbox send FramebufferCmd

Response = FramebufferResp(
    FramebufferResp {
        phy_width: 240,
        phy_height: 240,
        pitch: 960,
        pixels_ptr: 0x000000003c100000
    }
)

QEMU: Terminated
```
