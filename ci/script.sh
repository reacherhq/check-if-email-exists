# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release

    # TODO run with a $TEST_EMAIL, to test that the tool actually works. Right
    # now, Travis blocks ports 25, 587 and 465, so we cannot do that.
    cross run --target $TARGET $TEST_EMAIL
    cross run --target $TARGET --release $TEST_EMAIL
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
