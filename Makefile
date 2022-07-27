.PHONY: format
format:
	command cargo +nightly fmt

.PHONY: udeps
udeps:
	command cargo +nightly udeps
