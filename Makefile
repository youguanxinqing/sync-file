.PHONY: precommit fmt release

precommit:
	sh .pre-commit.sh

fmt:
	cargo fmt

release:
	cargo build --release
