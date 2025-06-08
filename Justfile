# Default recipe
default:
    @just --list

run-reloading *ARGS="":
    @echo "Building binary and dylib..."
    cargo runcc -c .cargo/runcc-build.yaml
    @echo "Watching for changes and launching program..."
    cargo runcc -c .cargo/runcc-run.yaml

watch:
    @echo "Watching for changes..."
    cargo watch -w iced_ui -d 0.01 -x "rustc --package iced_ui --crate-type dylib --profile reload --features reload -- -C link-arg=-Wl,--whole-archive"

run:
    @echo "Launching program..."
    cargo run --profile reload --features reload --target-dir target-bin

experiment:
    cargo rustc --profile reload --package deps --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,--whole-archive -C link-dead-code #-C link-args=-Wl,--export-dynamic
    cargo rustc --profile reload --package types --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package store --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code -l static=sqlcipher -L native=$(find target/reload/build -name "libsqlite3-sys-*" -type d | head -1)/out
    cargo rustc --profile reload --package handles --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package font_and_icons --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package wallet --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynbutton --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dyncontainer --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynpick_list --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynrule --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynscrollable --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dyntext --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dyntext_input --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package widgets --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package styles --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --features reload --package iced_ui --crate-type cdylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --features reload --package mercurium -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic
    cp .cargo/libstd-b38d63f8b721b18d.so target/reload
    cd target/reload && LD_LIBRARY_PATH=. ./mercurium

iced-ui:
    cargo rustc --package deps --crate-type cdylib #-- -L dependency=. -C link-arg=-Wl,--whole-archive -C link-dead-code -C link-args=-Wl,--export-dynamic
    cargo rustc --package iced_ui --crate-type cdylib -- -C prefer-dynamic

bin:
    cargo rustc --package mercurium -- -C link-args=-Wl,--export-dynamic
