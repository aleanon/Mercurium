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
    cargo watch -w iced_ui -x "rustc --package iced_ui --crate-type dylib --profile reload --features reload -- -C prefer-dynamic"

run:
    @echo "Launching program..."
    cargo run --profile reload --features reload --target-dir target-bin

experiment:
    cargo rustc --profile reload --package deps --crate-type dylib -- -L dependency=. -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package types --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package store --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package handles --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package font_and_icons --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package wallet --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --features reload --package iced_ui --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package styles --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package widgets --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynbutton --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dyncontainer --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynpick_list --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynrule --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dynscrollable --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dyntext --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --package dyntext_input --crate-type dylib -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic -C link-arg=-Wl,--whole-archive -C link-dead-code
    cargo rustc --profile reload --features reload --package mercurium -- -L dependency=. -C link-arg=-Wl,-rpath,'$ORIGIN' -C prefer-dynamic
    cp .cargo/libstd-b38d63f8b721b18d.so target/reload
