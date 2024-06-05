# Benchmark

## OS hosting DB
```console
grep PRETTY_NAME /etc/os-release
```
```
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
```
```console
uname -a
```
```
Linux DESKTOP-U9VGCHS 5.15.146.1-microsoft-standard-WSL2 #1 SMP Thu Jan 11 04:09:03 UTC 2024 x86_64 GNU/Linux
```


## Hardware hosting DB
```console
lscpu
```
```
Architecture:            x86_64
  CPU op-mode(s):        32-bit, 64-bit
  Address sizes:         36 bits physical, 48 bits virtual
  Byte Order:            Little Endian
<REDACTED>
CPU(s):                  4
<REDACTED>
  Model name:            Intel(R) Core(TM) i5-2410M CPU @ 2.30GHz
<REDACTED>
```
```console
lsmem | grep -i 'total online'
```
```
Total online memory:       4G
```

Block device: Samsung SSD 850 PRO 256GB

## Setup
```
docker compose up db_bench
just db-bootstrap db-migrate
./benchmark/bench.sh
```
