NIX_DEV_PROFILE := .dev

.PHONY: dev
dev:
	nix develop --profile $(NIX_DEV_PROFILE)

.PHONY: test
test:
	nix develop --profile $(NIX_DEV_PROFILE) -c cargo test
