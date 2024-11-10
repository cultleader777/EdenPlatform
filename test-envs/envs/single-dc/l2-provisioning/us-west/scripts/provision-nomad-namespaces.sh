#!/bin/sh
while ! nomad status &> /dev/null
do
  echo Nomad not ready yet, sleeping...
  sleep 3
done
nomad namespace apply -description "Eden platform" epl
nomad namespace apply -description "ze apps" apps
nomad namespace apply -description "ze system" system
