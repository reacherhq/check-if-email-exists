# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --all --target $TARGET
    cross test --all --target $TARGET --release

    # Run the binary with a $TEST_EMAIL, to test that the tool actually works.
    # aka some quick e2e test on the binary itself
    # TODO This only works on osx now, see #11
    if [ $TRAVIS_OS_NAME = osx ]; then
        cross run --target $TARGET $TEST_EMAIL | grep "\"is_deliverable\": true"
        cross run --target $TARGET --release $TEST_EMAIL | grep "\"is_deliverable\": true"
    fi
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
