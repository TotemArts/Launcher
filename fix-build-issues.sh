#!/bin/bash
cd "$( ls -1tqd ~/.cargo/git/checkouts/xdelta-*/*/ | head -n 1 )" || exit
if [ $( ls -a1tq $(pwd)/lzma-sys/xz/ | wc -l ) == "3" ]
then
  rm -Rf lzma-sys/xz/
  git submodule update --init --recursive
else
  git submodule foreach --recursive git fetch
  git submodule foreach git merge origin master
  git submodule update --init --recursive
fi
