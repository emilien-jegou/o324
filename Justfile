start-dev-daemon:
	cargo watch -x 'run --bin "o324-daemon" -- --config ./examples/demo-config.toml start'

build:
	cargo build

clear:
	rm -rf ./target

clear-all: _confirm_prompt clear
	rm -rf ./packages

@_confirm_prompt:
	while true; do \
	    read -p "Are you sure? [y/N] " answer; \
	    case $answer in \
	        [yY]) break;; \
	        [nN]|"") echo "Aborting."; exit 1;; \
	        *) echo "Invalid answer. Please enter 'y' or 'n'.";; \
	    esac \
	done

# regenerate the .versio.yaml file
generate-versio-config:
	cd scripts && npm install && cd ..
	./scripts/generate-versio-config

versio COMMAND: generate-versio-config
	versio -m local -x local "{{COMMAND}}"

release:
	just versio release
