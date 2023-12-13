build:
	cargo build

watch CRATE:
	cargo-watch -x 'build --color=always -p {{CRATE}}'

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
regenerate-versio-config:
	./scripts/regenerate-versio-config

versio COMMAND:
	versio -m local -x local "{{COMMAND}}"

release:
	just versio release
