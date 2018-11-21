# Graphics Example

Example graphics using GPU memory.

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

Add `--graphic=` to get the display, `--nographic` is supplied by default.

```bash
cargo fel4 simulate
```

```text
qemu-system-aarch64 -machine raspi3   -s -serial null -serial mon:stdio -m size=1024M  -kernel debug/kernel -initrd debug/feL4img

ELF-loader started on CPU: ARM Ltd. Cortex-A53 r0p4
  paddr=[80000..24bfff]
ELF-loading image 'kernel'
  paddr=[1000000..122ffff]
  vaddr=[ffffff8000000000..ffffff800022ffff]
  virt_entry=ffffff8000000000
ELF-loading image 'root-task'
  paddr=[1230000..1270fff]
  vaddr=[400000..440fff]
  virt_entry=400148
Enabling MMU and paging
Jumping to kernel-image entry point...

Warning:  gpt_cnt_tval 62500000, expected 19200000
Bootstrapping kernel
Booting all finished, dropped to user space
Mapped VideoCore Mailbox device region
  vaddr = 0x10000880 paddr = 0x3F00B880
  base vaddr = 0x10000000 base_paddr 0x3F00B000
Allocated pmem page
  vaddr = 0x10001000 paddr = 0x1274000

Creating a Display

FramebufferResp {
    phy_width: 800,
    phy_height: 480,
    pitch: 3200,
    paddr: 1007681536
}
```
