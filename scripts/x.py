import json
import subprocess
import time


def send_message(proc, message):
    data = json.dumps(message).encode("utf-8")
    header = f"Content-Length: {len(data)}\r\n\r\n".encode("utf-8")
    proc.stdin.write(header)
    proc.stdin.write(data)
    proc.stdin.flush()


def read_message(proc):
    header = b""
    while b"\r\n\r\n" not in header:
        chunk = proc.stdout.read(1)
        if not chunk:
            raise EOFError("Unexpected EOF while reading header")
        header += chunk
    header_text = header.decode("utf-8")
    content_length = None
    for line in header_text.split("\r\n"):
        if line.lower().startswith("content-length:"):
            content_length = int(line.split(":", 1)[1].strip())
            break
    if content_length is None:
        raise ValueError("Missing Content-Length header")
    body = proc.stdout.read(content_length)
    return json.loads(body.decode("utf-8"))


def main():
    proc = subprocess.Popen(
        ["simple-completion-language-server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # 1. Initialize the server.
    initialize = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {"capabilities": {}},
    }
    send_message(proc, initialize)
    init_resp = read_message(proc)
    print("Initialize response:", init_resp)

    # 2. Open a document.
    did_open = {
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file://dummy",
                "languageId": "plaintext",
                "version": 1,
                "text": "hello world",
            }
        },
    }
    send_message(proc, did_open)
    time.sleep(0.2)

    # 3. Send a completion request.
    completion = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/completion",
        "params": {
            "textDocument": {"uri": "file://dummy"},
            "position": {"line": 0, "character": 3},
        },
    }
    send_message(proc, completion)
    completion_resp = read_message(proc)
    print("Completion response:", completion_resp)

    # 4. Send a shutdown request without params.
    shutdown = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "shutdown",
    }
    send_message(proc, shutdown)
    shutdown_resp = read_message(proc)
    print("Shutdown response:", shutdown_resp)

    # 5. Send an exit notification.
    exit_notification = {"jsonrpc": "2.0", "id": 3, "method": "exit"}
    send_message(proc, exit_notification)
    completion_resp = read_message(proc)
    print("Completion response:", completion_resp)
    proc.wait()


if __name__ == "__main__":
    main()
