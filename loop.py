import os
import time
import subprocess
import tempfile
t = time.time()
while True:
#	try:
		print(time.time()-t)
		t = time.time()
		with tempfile.TemporaryFile() as tempf:

			proc = subprocess.Popen(["sh","./run.sh"], stdout=tempf)
			proc.wait()

			proc.seek(0)
			content = tempf.read().replace("\"","\\\"")
			print(content)
			os.system(f"gh release edit test -n \"{content}\"")
		#os.system("./run.sh")
		time.sleep(1)
#	except:
		print("error")
		pass #someone pressed ctrl+c or the internet died, use zz thingy
