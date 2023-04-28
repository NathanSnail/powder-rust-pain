git fetch
ping -c 2 127.0.0.1 > a.txt
git pull
ping -c 2 127.0.0.1 > a.txt
cargo build --target x86_64-pc-windows-gnu > log.txt
ping -c 2 127.0.0.1 > a.txt
gh release upload --clobber test target/x86_64-pc-windows-gnu/debug/new.exe
ping -c 2 127.0.0.1 > a.txt
