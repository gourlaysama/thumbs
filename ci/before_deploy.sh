#!/usr/bin/env bash
set -ex

main () {
	local src=$(pwd) \
		stage=$(mktemp -d)

	cp target/release/thumbs $stage/
	strip $stage/thumbs

	mkdir -p deploy
	cd $stage
	tar czf $src/deploy/thumbs-$TRAVIS_TAG-x86_64-unknown-linux-gnu.tar.gz *
	cd $src

	rm -rf $stage
}

main
