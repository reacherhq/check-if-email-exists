# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    # FIXME This only works on osx now
    # See https://github.com/amaurymartiny/check-if-email-exists/issues/11
    if [ $TRAVIS_OS_NAME = osx ]; then
        cross test --all --target $TARGET
        cross test --all --target $TARGET --release
    fi

    cross run --target $TARGET
    cross run --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
