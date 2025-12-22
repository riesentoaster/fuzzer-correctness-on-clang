export F_OUT_DIR=nautilus-2
export F_CORES="24-27"
export F_PORT="6"
just fuzzer_lib preloads
rm -rf out/$F_OUT_DIR
mkdir -p out/$F_OUT_DIR
screen -S $F_OUT_DIR -L -Logfile out/$F_OUT_DIR/screen.log