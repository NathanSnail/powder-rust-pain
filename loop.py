import os
import time
t = time.time()
while True:
    print(time.time()-t)
    t = time.time()
    os.system("./run.sh")
    time.sleep(1)
    print("error")
