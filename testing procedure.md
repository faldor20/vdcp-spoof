

https://stackoverflow.com/questions/52187/virtual-serial-port-for-linux


You can test socat to create Virtual Serial Port doing the following procedure (tested on Ubuntu 12.04):

Open a terminal (let's call it Terminal 0) and execute it:

socat -d -d pty,raw,echo=0 pty,raw,echo=0

The code above returns:

2013/11/01 13:47:27 socat[2506] N PTY is /dev/pts/2
2013/11/01 13:47:27 socat[2506] N PTY is /dev/pts/3
2013/11/01 13:47:27 socat[2506] N starting data transfer loop with FDs [3,3] and [5,5]

Open another terminal and write (Terminal 1):

cat < /dev/pts/2

this command's port name can be changed according to the pc. it's depends on the previous output.

2013/11/01 13:47:27 socat[2506] N PTY is /dev/pts/**2**
2013/11/01 13:47:27 socat[2506] N PTY is /dev/pts/**3**
2013/11/01 13:47:27 socat[2506] N starting data transfer loop with FDs 

you should use the number available on highlighted area.

Open another terminal and write (Terminal 2):

echo "Test" > /dev/pts/3

Now back to Terminal 1 and you'll see the string "Test".
