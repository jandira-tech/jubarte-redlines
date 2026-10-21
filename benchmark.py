import time
import re
from pathlib import Path
import sys

# Add scripts dir to path to import cli_id_sources
sys.path.append('scripts')
import cli_id_sources

def run_benchmark():
    # Setup
    root = Path('.')
    fns = cli_id_sources.functions(root)

    start_time = time.time()
    for _ in range(10): # Run a few times to get measurable time
        for fn in fns.values():
            cli_id_sources.messages(fn, root)
    end_time = time.time()
    print(f"Original messages execution time: {end_time - start_time:.4f} seconds")

run_benchmark()
