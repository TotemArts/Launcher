#!/bin/bash
cd "$(ls -1tqd ~/.cargo)" || exit
if [ ! -f "config" ]
then
  echo "[target.x86_64-pc-windows-gnu]
linker = \"x86_64-w64-mingw32-gcc\"
ar = \"x86_64-w64-mingw32-gcc-ar\"" > config
fi
cd "$( ls -1tqd ~/.cargo/git/checkouts/xdelta-*/*/ | head -n 1 )" || exit
if [ $( ls -a1tq $(pwd)/lzma-sys/xz/ | wc -l ) == "3" ]
then
  rm -Rf lzma-sys/xz/
  git submodule update --init --recursive
else
  git submodule update --init --recursive
fi
