git fetch
ping -c 4 127.0.0.1 > a.txt
git reset --hard origin/main
ping -c 4 127.0.0.1 > a.txt
git fetch
ping -c 4 127.0.0.1 > a.txt
cargo build --target x86_64-pc-windows-gnu
ping -c 4 127.0.0.1 > a.txt
#git add *
#git commit -m "yes"
ping -c 4 127.0.0.1 > a.txt
gh release upload --clobber test target/x86_64-pc-windows-gnu/debug/new.exe
