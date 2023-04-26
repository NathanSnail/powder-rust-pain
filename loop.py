import os
import time
t = time.time()
while True:
	try:
	
		print(time.time()-t)
		t = time.time()
		os.system("./run.sh")
		time.sleep(2)
	except:
		pass #someone pressed ctrl+c or the internet died, use zz thingy
