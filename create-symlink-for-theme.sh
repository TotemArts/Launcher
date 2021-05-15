#!/bin/bash
echo "Name theme to create symlink for:"
read theme
if [ $( ls -1tqd "$(pwd)/$theme" | wc -l ) == "1" ]
then
  ln -s "$(pwd)/$theme" "$(pwd)/target/release/$theme" 2>/dev/null
  ln -s "$(pwd)/$theme" "$(pwd)/target/debug/$theme" 2>/dev/null
else
  echo "There is no theme named \"$theme\" in your current folder"
fi
