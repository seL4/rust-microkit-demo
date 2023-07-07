# Simple test script
import pexpect


# Start QEMU
child = pexpect.spawn('make run',encoding='utf-8')
fout = open('log.txt','w')
child.logfile = fout
# Wait for the prompt
child.expect('banscii>',timeout=1)
# Try hello world
child.sendline('Hello World\r')
# Wait for the prompt (ignore the output)
child.expect('banscii>',timeout=1)
# Escape sequence
child.sendcontrol('A')
child.send('x')
# Termination confirmation
child.expect('QEMU: Terminated',timeout=1)
