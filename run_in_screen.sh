export F_OUT_DIR=fandango-interspersedhavoc-seedless
export F_CORES="36-39"
export F_PORT="9"

# Copy LLVM directory to ramdisk if not already present
RAMDISK_LLVM="/dev/shm/llvm"
if [ ! -d "$RAMDISK_LLVM" ] || [ ! -f "$RAMDISK_LLVM/README.md" ]; then
    echo "Copying LLVM directory to ramdisk at $RAMDISK_LLVM..."
    if [ -d "llvm" ]; then
        rm -rf "$RAMDISK_LLVM"
        cp -r llvm "$RAMDISK_LLVM"
        echo "LLVM directory copied to ramdisk successfully."
    else
        echo "Warning: llvm directory not found. Skipping ramdisk copy."
        exit 1
    fi
else
    echo "LLVM directory already exists in ramdisk at $RAMDISK_LLVM"
fi

REDIRECTION_SHARED_LIBRARY="/dev/shm/setup_guard_redirection/libsetup_guard_redirection.so"
if [ ! -f "$REDIRECTION_SHARED_LIBRARY" ]; then
    echo "Copying redirection shared library to ramdisk at $REDIRECTION_SHARED_LIBRARY..."
    mkdir -p $(dirname "$REDIRECTION_SHARED_LIBRARY")
    cp target/release/libsetup_guard_redirection.so "$REDIRECTION_SHARED_LIBRARY"
    echo "Redirection shared library copied to ramdisk successfully."
else
    echo "Redirection shared library already exists in ramdisk at $REDIRECTION_SHARED_LIBRARY"
fi

export TARGET_BINARY="$RAMDISK_LLVM/build/bin/clang"
export REDIRECTION_SHARED_LIBRARY="$REDIRECTION_SHARED_LIBRARY"

just fuzzer preloads
rm -rf out/$F_OUT_DIR
mkdir -p out/$F_OUT_DIR
screen -S $F_OUT_DIR -L -Logfile out/$F_OUT_DIR/screen.log