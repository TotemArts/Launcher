#!/bin/bash
echo Name theme to create symlink for:
read theme
if [ $( ls -1tqd ./$theme | wc -l ) == "1" ]
then
  ln -s "./$theme" "./target/release/$theme"
  ln -s "./$theme" "./target/debug/$theme"
else
  echo There is no theme named \"$theme\" in your current folder
fi
