#!/bin/bash
cd "$( ls -1tqd ~/.cargo/git/checkouts/xdelta-*/*/ | head -n 1 )" || exit
if [ $( ls -1tq ./lzma-sys/xz/ | wc -l ) == "1" ]
then
  rm -rf /lzma-sys/xz/
  git submodule update --init --recursive
else
  git submodule foreach --recursive git fetch
  git submodule foreach git merge origin master
  git submodule update --init --recursive
fi
