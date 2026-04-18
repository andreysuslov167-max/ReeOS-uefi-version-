# Hello, this instruction is for testing my OS.
## Download the open source code from the latest release (1.0.6)
Next we give access to run the startup code
```bash
chmod +x run.sh
./run.sh

```
Congratulations on the launch of the OS
Please note that you need to run it qemu
You can find out how to download it on the Internet.
You will also have to download the UEFI firmware
like this
```bash
wget https://retrage.github.io/edk2-nightly/bin/RELEASEX64_OVMF.fd
mv RELEASEX64_OVMF.fd OVMF.fd
```
but this is not necessary if everything works for you without downloading
