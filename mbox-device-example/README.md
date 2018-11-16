# Hello World

A hello world `no_std` Rust app running on the seL4 microkernel for the Raspberry Pi 3.

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
ELF-loader started on CPU: ARM Ltd. Cortex-A53 r0p4
  paddr=[80000..1abfff]
ELF-loading image 'kernel'
  paddr=[1000000..122ffff]
  vaddr=[ffffff8000000000..ffffff800022ffff]
  virt_entry=ffffff8000000000
ELF-loading image 'root-task'
  paddr=[1230000..1278fff]
  vaddr=[400000..448fff]
  virt_entry=4004d0
Enabling MMU and paging
Jumping to kernel-image entry point...

Warning:  gpt_cnt_tval 62500000, expected 19200000
Bootstrapping kernel
Booting all finished, dropped to user space

hello from a feL4 app!

Caught cap fault in send phase at address (nil)
while trying to handle:
vm fault on code at address (nil) with status 0x82000006
in thread 0xffffff8000000400 "child of: 'rootserver'" at address (nil)
```
