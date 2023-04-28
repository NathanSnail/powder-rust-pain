git fetch
ping -c 2 127.0.0.1 > a.txt
git pull
ping -c 2 127.0.0.1 > a.txt
cargo build --target x86_64-pc-windows-gnu > log.txt
gh release upload --clobber test target/x86_64-pc-windows-gnu/debug/new.exe
pinc -c 10 127.0.0.1 > a.txt