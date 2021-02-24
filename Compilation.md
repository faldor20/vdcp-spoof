# Cross compilation:
## NIX
run the nix shell and then compile. if you immidietly get an error about ptrheads, remove pthreads from the cross-Shell.nix file buildinputs
you will then get an error a the end of the build. add it back in and it will work.
## Docker
compile the docker file. in ``./cross-compilation`` using ``./cross-compilation/build.sh``
run ./Buildwin.sh
## Docker development env.:
open the docker visual studio code dev envirnment thing. 
you're good to go.
