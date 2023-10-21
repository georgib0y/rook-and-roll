import asyncio
import subprocess

ENGINE_PROCESS_NAME="uci"

STDIN = 0
STDOUT = 1

STD_NAMES = ["stdin", "stdout"]

async def attatch_to_fd(pid: str, fd: int):
  process = await asyncio.create_subprocess_shell(f"tail -f /proc/{pid}/fd/{fd}",
                                                  stdout=asyncio.subprocess.PIPE)
  
  while True:
    line = await process.stdout.readline()
    print(f"[{pid}: {STD_NAMES[fd]}] {line}")

async def attatch_to_process(pid: str):
  await asyncio.gather(attatch_to_fd(pid, STDIN), attatch_to_fd(pid, STDOUT))

async def main():
  pids = subprocess.run(["pidof", ENGINE_PROCESS_NAME], capture_output=True).stdout.decode().strip()
  print(pids)

  if pids == "":
    print("Engine not running")
    return 
  
  await asyncio.gather(*(attatch_to_process(pid) for pid in pids.split(" ")))

asyncio.run(main())
