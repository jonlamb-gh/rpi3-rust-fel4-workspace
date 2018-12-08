# Example project, render a simple clock

## Building

```bash
cargo fel4 build
```

## Running

```bash
U-Boot> run bootfel4
Waiting for Ethernet connection... done.
Using smsc95xx_eth device
Filename 'feL4img'.
Load address: 0x20000000
Loading: ##################################################  1.9 MiB
         2.4 MiB/s
done
Bytes transferred = 2021844 (1ed9d4 hex)
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
Mapping VideoCore mailbox device
  dev_pmem base_vaddr = 0x10000000 base_paddr = 0x3F00B000 size = 0x1000 offset = 0x880
Mapping DMA device
  dev_pmem base_vaddr = 0x10001000 base_paddr = 0x3F007000 size = 0x1000 offset = 0x0
Reserving DMA pool size 0x179000 (1544192), 378 pages
  dma_pmem vaddr = 0x10002000 paddr = 0x6C00000 size = 0x17A000
Mailbox buffer DMA pmem
  vaddr = 0x10002000 paddr = 0x6C00000 size = 0x1000
Display scratchpad DMA pmem
  vaddr = 0x10003000 paddr = 0x6C01000 size = 0x1000
Display backbuffer DMA pmem
  vaddr = 0x10004000 paddr = 0x6C02000 size = 0x177000
Mapped in VideoCore GPU pmem
  vaddr = 0x1017C000 paddr = 0xFEA83000 size = 0x177000

Render thread running
```

