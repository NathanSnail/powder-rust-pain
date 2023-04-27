cargo build --target x86_64-pc-windows-gnu > log.txt
gh release edit test -n "$1"
gh release upload --clobber test target/x86_64-pc-windows-gnu/debug/new.exe
