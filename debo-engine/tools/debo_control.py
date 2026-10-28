#!/usr/bin/env python3
"""Small standard-library controller for the DEBO face protocol v1."""
import argparse
import json
import socket
import sys

HOST = "127.0.0.1"
PORT = 45821


def make_message(args):
    if args.command == "expression":
        if args.value not in {"neutral", "curious", "happy", "surprised", "sleepy"}:
            raise ValueError("expression must be neutral, curious, happy, surprised, or sleepy")
        return {"version": 1, "command": "set_expression", "value": args.value}
    if args.command == "eye-color":
        if args.value.lower() not in {"cyan", "blue", "purple", "green", "amber", "pink"}:
            raise ValueError("eye colour must be cyan, blue, purple, green, amber, or pink")
        return {"version": 1, "command": "set_eye_color", "value": args.value.lower()}
    if args.command == "gaze":
        if not (-1 <= args.x <= 1 and -1 <= args.y <= 1):
            raise ValueError("gaze x and y must each be between -1 and 1")
        return {"version": 1, "command": "gaze", "x": args.x, "y": args.y}
    if args.command == "blink":
        return {"version": 1, "command": "blink"}
    return {"version": 1, "command": "centre_gaze"}


def main():
    parser = argparse.ArgumentParser(description="Send a command to the running DEBO face preview")
    sub = parser.add_subparsers(dest="command", required=True)
    expression = sub.add_parser("expression", help="set the face expression")
    expression.add_argument("value", choices=["neutral", "curious", "happy", "surprised", "sleepy"])
    colour = sub.add_parser("eye-color", help="set the eye colour")
    colour.add_argument("value", help="cyan, blue, purple, green, amber, or pink")
    gaze = sub.add_parser("gaze", help="set gaze coordinates in the range -1..1")
    gaze.add_argument("x", type=float)
    gaze.add_argument("y", type=float)
    sub.add_parser("blink", help="trigger a blink")
    sub.add_parser("centre", help="centre the gaze")
    args = parser.parse_args()

    try:
        message = make_message(args)
        with socket.create_connection((HOST, PORT), timeout=3) as sock:
            sock.sendall((json.dumps(message, separators=(",", ":")) + "\n").encode("utf-8"))
            with sock.makefile("r", encoding="utf-8") as response_file:
                response = json.loads(response_file.readline())
        print(json.dumps(response, indent=2))
        if not response.get("accepted"):
            return 1
        return 0
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"DEBO command failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
