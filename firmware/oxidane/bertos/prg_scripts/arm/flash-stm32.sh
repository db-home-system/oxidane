#!/bin/bash

echo "Flash $IMAGE_FILE on Nucleo Board.."
echo
st-flash write $IMAGE_FILE 0x8000000
echo
echo

