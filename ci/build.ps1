$cross = $args[0]
$target_triple = $args[1]
$release_build = $args[2]

# reacher_worker doesn't compile on windows because of this bug:
# https://github.com/amqp-rs/reactor-trait/issues/1
# So we only build the check_if_email_exists binary for Windows.
if ( $release_build -ne "RELEASE" ) {
    Invoke-Expression "$cross build --bin check_if_email_exists --target $target_triple"
    Invoke-Expression "$cross build --bin check_if_email_exists --target $target_triple --all-features"
} else {
    Invoke-Expression "$cross build --bin check_if_email_exists --target $target_triple --all-features --release"
}