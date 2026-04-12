#!/bin/bash


cargo build --release --target x86_64-unknown-uefi || exit 1

mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/release/reeos-uefi.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 \
    -bios OVMF.fd \
    -drive file=fat:rw:esp,format=raw,media=disk \
    -m 128M \
    -serial stdio