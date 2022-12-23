cd backend
cargo update
cargo build --release
cd ..
echo F|xcopy /Y backend\target\release\renegade-x-launcher.exe RenegadeX-Launcher.exe
pause