docker run -v "$(pwd):/project:z" -w /project -it faldor20/rust-cross-comp  cargo +nightly build --release --target x86_64-pc-windows-gnu
#docker run -v /var/run/docker.sock:/var/run/docker.sock -v "$(pwd):/project" \
#  -w /project -it faldor20/rust-cross  cross build --release --target x86_64-pc-windows-gnu