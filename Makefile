ROBBOT_VERSION := `git describe --tags`
ROBBOT_BUILT := `date -u +%FT%T%z`

CARGO := cargo

build:
	ROBBOT_VERSION=${ROBBOT_VERSION} ROBBOT_BUILT=${ROBBOT_BUILT} $(CARGO) build --release
