export PYTHONPATH=$(echo .venv/lib/python*/site-packages)

./target/release/libafl_nautilus_fuzzer \
	--grammar-file c.json \
	--output "out/$F_OUT_DIR" \
	--stdout-file /dev/null \
	--stderr-file /dev/null \
	--cores $F_CORES \
	--broker-port "133$F_PORT"
