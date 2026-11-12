import unittest

import run
import search


class SearchScript(unittest.TestCase):
    def test_a_search_types_the_query_presses_enter_then_jumps(self):
        names = [name for name, _ in search.script("ab")]

        self.assertEqual(names[0], "/")
        self.assertEqual(names[3], "Enter")
        self.assertEqual(names[4:6], ["n", "N"])

    def test_enter_has_a_200_ms_budget_and_any_other_key_16_ms(self):
        failures = run.search_over_budget([("Enter", 150.0), ("n", 15.0)])
        self.assertEqual(failures, [])

        failures = run.search_over_budget([("Enter", 250.0), ("n", 20.0), ("N", None)])
        self.assertEqual(len(failures), 3)
        self.assertIn("Enter", failures[0])
        self.assertIn("over 200 ms", failures[0])
        self.assertIn("over 16 ms", failures[1])


if __name__ == "__main__":
    unittest.main()
