#!/bin/bash

TAG=3.4.0
if [ ! -d ./sleef ]; then 
    git clone -b $TAG https://github.com/shibatch/sleef.git
fi

LLVM_VERSION=`llvm-config --version | cut -d . -f 1,2`
CLANG=clang-$LLVM_VERSION
if ! type $CLANG > /dev/null 2>&1 ; then
    echo "$CLANG does not exist."
    exit 1
fi

# cmake
cd sleef
mkdir -p build
cd build
cmake -DSLEEF_ENABLE_LLVM_BITCODE=1 \
      -DCLANG_EXE_PATH=$CLANG ..
make -j8 llvm-bitcode

cd lib

for llfile in `ls *.ll`
do 
    `llvm-config --bindir`/llvm-as $llfile
done
