git fetch
cargo build --target x86_64-pc-windows-gnu
#git add *
#git commit -m "yes"
gh release upload --clobber test target/x86_64-pc-windows-gnu/debug/new.exe
