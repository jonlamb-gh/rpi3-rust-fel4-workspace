# Graphics Example

Example graphics using GPU memory.

![screenshot](https://jonlamb-gh.github.io/rpi3-fel4/rpi3_gui_ex.png)

## Building

```bash
cargo fel4 build
```

## Simulating

Add `--graphic=` to get the display, `--nographic` is supplied by default.

```bash
cargo fel4 simulate
```

## Running

```text
U-Boot> run bootfel4
Waiting for Ethernet connection... done.
Using smsc95xx_eth device
Filename 'feL4img'.
Load address: 0x20000000
Loading: ##################################################  1.9 MiB
         2.4 MiB/s
done
Bytes transferred = 2030548 (1efbd4 hex)
## Starting application at 0x20000000 ...

ELF-loader started on CPU: ARM Ltd. Cortex-A53 r0p4
  paddr=[20000000..201f7fff]
ELF-loading image 'kernel'
  paddr=[1000000..122ffff]
  vaddr=[ffffff8000000000..ffffff800022ffff]
  virt_entry=ffffff8000000000
ELF-loading image 'root-task'
  paddr=[1230000..1290fff]
  vaddr=[400000..460fff]
  virt_entry=400390
Enabling MMU and paging
Jumping to kernel-image entry point...

Bootstrapping kernel
Booting all finished, dropped to user space
Mapped VideoCore Mailbox device region
  vaddr = 0x10000880 paddr = 0x3F00B880
  base vaddr = 0x10000000 base_paddr 0x3F00B000
Allocated mbox buffer pmem page
  vaddr = 0x10001000 paddr = 0x1297000
Mapped DMA device region
  vaddr = 0x10002000 paddr = 0x3F007000
Allocated DMA control block(s) pmem page, holds 128 control blocks
  vaddr = 0x10003000 paddr = 0x129F000

Requesting framebuffer

FramebufferResp {
    phy_width: 600,
    phy_height: 300,
    pitch: 2432,
    pixel_order: BGR,
    bus_paddr: 4273233920,
    paddr: 1052008448
}
Allocated display backbuffer
  vaddr = 0x100B7000 paddr = 0x1300000

Render thread running

ThreadData {
    dma_vaddr: 268443648,
    scratchpad_vaddr: 268447744,
    scratchpad_paddr: 19525632,
    fb_width: 600,
    fb_height: 300,
    fb_pitch: 2432,
    fb_pixel_order: BGR,
    fb_vaddr: 268451840,
    fb_paddr: 4273233920,
    fb_backbuffer_vaddr: 269185024,
    fb_backbuffer_paddr: 19922944
}
```
