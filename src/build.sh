
rm -rf esp
rm -f reeos.img


if [ ! -f "target/x86_64-unknown-uefi/release/reeos-uefi.efi" ]; then
    echo "❌ EFI file not found! Building..."
    cargo build --release --target x86_64-unknown-uefi
fi


mkdir -p esp/efi/boot


if cp target/x86_64-unknown-uefi/release/reeos-uefi.efi esp/efi/boot/bootx64.efi; then
    echo "✅ EFI file copied"
else
    echo "❌ Failed to copy EFI file"
    exit 1
fi


if [ ! -f "esp/efi/boot/bootx64.efi" ]; then
    echo "❌ bootx64.efi not found in esp/efi/boot/"
    exit 1
fi


dd if=/dev/zero of=reeos.img bs=1M count=64
mkfs.vfat reeos.img


mmd -i reeos.img ::efi
mmd -i reeos.img ::efi/boot
mcopy -i reeos.img esp/efi/boot/bootx64.efi ::efi/boot/

echo "✅ Image created: reeos.img"
echo ""
echo "Run with:"
echo "qemu-system-x86_64 -bios OVMF.fd -drive format=raw,file=reeos.img"