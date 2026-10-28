import importlib.util
import pathlib
import unittest

MODULE_PATH = pathlib.Path(__file__).with_name("debo_control.py")
SPEC = importlib.util.spec_from_file_location("debo_control", MODULE_PATH)
controller = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(controller)


class ControllerMessageTests(unittest.TestCase):
    def test_expression_message_is_versioned(self):
        args = type("Args", (), {"command": "expression", "value": "happy"})()
        self.assertEqual(controller.make_message(args), {"version": 1, "command": "set_expression", "value": "happy"})

    def test_colour_is_normalized(self):
        args = type("Args", (), {"command": "eye-color", "value": "PuRpLe"})()
        self.assertEqual(controller.make_message(args)["value"], "purple")

    def test_gaze_outside_bounds_is_rejected(self):
        args = type("Args", (), {"command": "gaze", "x": 1.1, "y": 0.0})()
        with self.assertRaises(ValueError):
            controller.make_message(args)

    def test_blink_message(self):
        args = type("Args", (), {"command": "blink"})()
        self.assertEqual(controller.make_message(args), {"version": 1, "command": "blink"})


if __name__ == "__main__":
    unittest.main()
