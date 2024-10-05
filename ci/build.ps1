$cross = $args[0]
$target_triple = $args[1]
$release_build = $args[2]

# Only the CLI and is supported for now on Windows
if ( $release_build -ne "RELEASE" ) {
    Invoke-Expression "$cross build --bin check_if_email_exists --target $target_triple"
    Invoke-Expression "$cross build --bin check_if_email_exists --target $target_triple --all-features"
} else {
    Invoke-Expression "$cross build --bin check_if_email_exists --target $target_triple --all-features --release"
}