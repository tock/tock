python3 -m venv venv
source venv/bin/activate
pip install pyserial

# run differential testing
python diff_test.py >> diff_test_out.log
# run performance benchmarks
python benchmarks.py >> benchmark_out.log
# run memory benchmarks
python memory.py >> memory_out.log

# cleanup
rm -r ./venv
git checkout master
