export F_OUT_DIR=greybox-seed
export F_CORES="36-39"
export F_PORT="9"
just fuzzer preloads
rm -rf out/$F_OUT_DIR
mkdir -p out/$F_OUT_DIR
screen -S $F_OUT_DIR -L -Logfile out/$F_OUT_DIR/screen.log