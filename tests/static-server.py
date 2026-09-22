#!/usr/bin/env python3
"""Serve exported test files using Python's standard handler on loopback."""
import faulthandler

faulthandler.dump_traceback_later(8)
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import socketserver
import sys


class LoopbackServer(ThreadingHTTPServer):
    def server_bind(self):
        # HTTPServer resolves a hostname here. This fixture uses a numeric local
        # address and needs no reverse DNS, which can stall hosted macOS runners.
        socketserver.TCPServer.server_bind(self)
        self.server_name = "localhost"
        self.server_port = self.server_address[1]


with LoopbackServer(("127.0.0.1", 0), partial(SimpleHTTPRequestHandler, directory=sys.argv[1])) as server:
    faulthandler.cancel_dump_traceback_later()
    print(f"http://127.0.0.1:{server.server_port}/", flush=True)
    server.serve_forever()
