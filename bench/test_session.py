import subprocess
import sys
import unittest

import session

ALLOCATE_THEN_FREE = """
import mmap, time
block = mmap.mmap(-1, 200 * 1024 * 1024)
for _ in range(200):
    block.write(b"x" * 1024 * 1024)
block.close()
print("freed", flush=True)
time.sleep(60)
"""


class Memory(unittest.TestCase):
    def test_memory_is_the_peak_of_the_process_not_its_current_size(self):
        process = subprocess.Popen([sys.executable, "-c", ALLOCATE_THEN_FREE], stdout=subprocess.PIPE, text=True)
        self.addCleanup(process.wait)
        self.addCleanup(process.kill)
        self.assertEqual(process.stdout.readline(), "freed\n")
        process.stdout.close()

        self.assertGreaterEqual(session.memory(process), 200)


if __name__ == "__main__":
    unittest.main()
