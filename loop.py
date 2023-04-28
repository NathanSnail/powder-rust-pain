import os
import time
t = time.time()
while True:
	try:
	
		print(time.time()-t)
		t = time.time()
		os.system("./run.sh")
		os.system("cargo build --target x86_64-pc-windows-gnu > log.txt")
		time.sleep(1)
		with open("log.txt","r") as f:
			os.system(f"gh release edit test -n \"{f.read().replace("\n","\\n")}\"")
		os.system("gh release upload --clobber test target/x86_64-pc-windows-gnu/debug/new.exe")
		time.sleep(1)
		
	except:
		pass #someone pressed ctrl+c or the internet died, use zz thingy
