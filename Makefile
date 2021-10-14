ROBBOT_VERSION := `git describe --tags`
ROBBOT_BUILT := `date -u +%FT%T%z`

CARGO := cargo

build:
	ROBBOT_VERSION=${ROBBOT_VERSION} ROBBOT_BUILT=${ROBBOT_BUILT} $(CARGO) build --release

# Build Robbot against the current version of libc6 that comes
# with the debian buster release.
debian-buster:
	docker run --rm --user "$$(id -u)":"$$(id -g)" -v "$$PWD":/usr/share/robbot -w /usr/share/robbot rust:buster make build

# Build Robbot against the current version of libc6 that comes
# with the debian bullseye release.
debian-bullseye:
	docker run --rm --user "$$(id -u)":"$$(id -g)" -v "$$PWD":/usr/share/robbot -w /usr/share/robbot rust:bullseye make build
