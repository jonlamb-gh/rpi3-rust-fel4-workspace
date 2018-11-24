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

Warning:  gpt_cnt_tval 62500000, expected 19200000
Bootstrapping kernel
Booting all finished, dropped to user space
------------- bootinfo -------------
bootinfo.empty.start = 96
bootinfo.empty.end = 524288
bootinfo.userImageFrames.start = 15
bootinfo.userImageFrames.end = 64
bootinfo.untyped.start = 64
bootinfo.untyped.end = 96
bootinfo.untypedList
  length = 32
  [0 | 64] paddr = 0x1000000 - size_bits = 16 - is_device = 0
  [1 | 65] paddr = 0x1261000 - size_bits = 12 - is_device = 0
  [2 | 66] paddr = 0x1262000 - size_bits = 13 - is_device = 0
  [3 | 67] paddr = 0x1264000 - size_bits = 14 - is_device = 0
  [4 | 68] paddr = 0x1268000 - size_bits = 15 - is_device = 0
  [5 | 69] paddr = 0x1270000 - size_bits = 16 - is_device = 0
  [6 | 70] paddr = 0x1280000 - size_bits = 19 - is_device = 0
  [7 | 71] paddr = 0x1300000 - size_bits = 20 - is_device = 0
  [8 | 72] paddr = 0x1400000 - size_bits = 22 - is_device = 0
  [9 | 73] paddr = 0x1800000 - size_bits = 23 - is_device = 0
  [10 | 74] paddr = 0x2000000 - size_bits = 24 - is_device = 0
  [11 | 75] paddr = 0x3000000 - size_bits = 25 - is_device = 0
  [12 | 76] paddr = 0x5000000 - size_bits = 24 - is_device = 0
  [13 | 77] paddr = 0x6000000 - size_bits = 23 - is_device = 0
  [14 | 78] paddr = 0x6800000 - size_bits = 22 - is_device = 0
  [15 | 79] paddr = 0x6C00000 - size_bits = 21 - is_device = 0
  [16 | 80] paddr = 0x6E00000 - size_bits = 20 - is_device = 0
  [17 | 81] paddr = 0x6F00000 - size_bits = 19 - is_device = 0
  [18 | 82] paddr = 0x6F80000 - size_bits = 18 - is_device = 0
  [19 | 83] paddr = 0x6FC0000 - size_bits = 17 - is_device = 0
  [20 | 84] paddr = 0x6FE0000 - size_bits = 16 - is_device = 0
  [21 | 85] paddr = 0x6FF0000 - size_bits = 14 - is_device = 0
  [22 | 86] paddr = 0x6FF4000 - size_bits = 13 - is_device = 0
  [23 | 87] paddr = 0x3F00B000 - size_bits = 12 - is_device = 1
  [24 | 88] paddr = 0x3F100000 - size_bits = 12 - is_device = 1
  [25 | 89] paddr = 0x3F104000 - size_bits = 12 - is_device = 1
  [26 | 90] paddr = 0x3F200000 - size_bits = 12 - is_device = 1
  [27 | 91] paddr = 0x3F201000 - size_bits = 12 - is_device = 1
  [28 | 92] paddr = 0x3F300000 - size_bits = 12 - is_device = 1
  [29 | 93] paddr = 0x3F980000 - size_bits = 12 - is_device = 1
  [30 | 94] paddr = 0x3F215000 - size_bits = 12 - is_device = 1
  [31 | 95] paddr = 0x3F00B000 - size_bits = 12 - is_device = 1
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
        serial_number: 0
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
