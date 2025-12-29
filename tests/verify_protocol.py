import subprocess

def run_test():
    proc = subprocess.Popen(
        ["./target/debug/pbrain-gomoku-ai"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )

    def send(cmd):
        print(f"Sending: {cmd}")
        proc.stdin.write(cmd + "\n")
        proc.stdin.flush()

    def expect(pattern):
        line = proc.stdout.readline()
        print(f"Received: {line.strip()}")
        if pattern not in line:
            print(f"FAILED: Expected '{pattern}', got '{line.strip()}'")
            return False
        return True

    try:
        send("START 20")
        if not expect("OK"):
            return

        send("INFO timeout_match 3000")
        send("ABOUT")
        if not expect("name="): 
            return

        send("BEGIN")
        line = proc.stdout.readline().strip()
        print(f"Received (BEGIN response): {line}")
        if "," not in line:
             print("FAILED: Expected coordinates")
             return

        send("TURN 10,10")
        line = proc.stdout.readline().strip()
        print(f"Received (TURN response): {line}")
        if "," not in line:
            print("FAILED: Expected coordinates")
            return

        send("BOARD")
        send("10,10,1")
        send("10,11,2")
        send("DONE")
        line = proc.stdout.readline().strip()
        print(f"Received (BOARD response): {line}")
        if "," not in line:
            print("FAILED: Expected coordinates")
            return
            
        send("END")
        proc.wait(timeout=1)
        print("PASSED")

    except Exception as e:
        print(f"Exception: {e}")
        proc.kill()

if __name__ == "__main__":
    run_test()
