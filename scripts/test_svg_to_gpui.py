"""Regression checks for SVG coordinate precision in generated Rust icons."""

import struct
import unittest

from svg_to_gpui import number


class NumberTests(unittest.TestCase):
    def test_excess_precision_is_removed(self):
        self.assertEqual(number(-705.4142456), "-705.41425")
        self.assertEqual(number(28.14244652), "28.142447")

    def test_coordinates_preserve_f32_values(self):
        for value in (-705.1110229, 27.86776924, 0.00764646, 1.23456789):
            with self.subTest(value=value):
                self.assertEqual(
                    struct.pack("!f", value),
                    struct.pack("!f", float(number(value))),
                )

    def test_integer_and_zero_literals_remain_floats(self):
        self.assertEqual(number(1), "1.0")
        self.assertEqual(number(20), "2e+01")
        self.assertEqual(number(0), "0.0")
        self.assertEqual(number(-0.0), "0.0")


if __name__ == "__main__":
    unittest.main()
