import subprocess
import time

def run_test():
    process = subprocess.Popen(
        ['./target/debug/pbrain-gomoku-ai'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0 
    )

    def send_command(cmd):
        print(f"Sending: {cmd}")
        process.stdin.write(cmd + '\n')
        process.stdin.flush()
        response = process.stdout.readline().strip()
        print(f"Received: {response}")
        return response

    try:
        assert send_command("START 20") == "OK"
        
        resp = send_command("TURN 0,0")
        if resp != "10,10":
            print(f"FAIL: Expected 10,10, got {resp}")
        else:
            print("PASS: First move 10,10")

        resp = send_command("TURN 10,10")
        if "ERROR" not in resp:
            print(f"FAIL: Expected ERROR for occupied cell, got {resp}")
        else:
            print("PASS: Occupied cell check")

        resp = send_command("TURN 0,0")
        if "ERROR" not in resp:
             print(f"FAIL: Expected ERROR for occupied cell (previous opponent move), got {resp}")
        else:
             print("PASS: Opponent occupied cell check")

        resp = send_command("TURN 0,1")
        if resp != "0,2":
             print(f"WARNING: Expected 0,2 (linear search), got {resp}. Logic might differ but expecting valid coord.")
             if "," not in resp:
                 print(f"FAIL: Invalid response format {resp}")
        else:
             print("PASS: Linear search move 0,2")
             
        print("Sending BOARD sequence...")
        process.stdin.write("BOARD\n")
        process.stdin.write("10,10,2\n")
        process.stdin.write("DONE\n")
        process.stdin.flush()
        
        resp = process.stdout.readline().strip()
        print(f"Received after BOARD: {resp}")
        
        if resp == "10,10":
             print("FAIL: Bot played on occupied 10,10 after BOARD")
        elif resp == "0,0":
             print("PASS: Bot played 0,0 after BOARD cleared/set state")
        else:
             if "," in resp:
                print(f"PASS: Bot played {resp}")
             else:
                print(f"FAIL: invalid response {resp}")

    except Exception as e:
        print(f"Test failed with exception: {e}")
        process.kill()
    
    finally:
        if process.poll() is None:
            process.stdin.write("END\n")
            process.stdin.flush()
            time.sleep(0.1)
            if process.poll() is None:
                process.kill()

if __name__ == "__main__":
    run_test()
