# Cross compilation:
## NIX
run the nix shell and then compile. if you immidietly get an error about ptrheads, remove pthreads from the cross-Shell.nix file buildinputs
you will then get an error a the end of the build. add it back in and it will work.
## Docker
compile the docker file. in ``./cross-compilation`` using ``./build.sh``
IMPORTANT!!! the file must be run from within that folder. you must cd inot ./cross-compilation
run ./Buildwin.sh
## Docker development env.:
open the docker visual studio code dev envirnment thing. 
you're good to go.
