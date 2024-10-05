$cross = $args[0]
$target_triple = $args[1]
$release_build = $args[2]

# Only the CLI and backend are supported for now on Windows
if ( $release_build -ne "RELEASE" ) {
    Invoke-Expression "$cross build -p cli -p backend --target $target_triple"
    Invoke-Expression "$cross build -p cli -p backend --target $target_triple --all-features"
} else {
    Invoke-Expression "$cross build -p cli -p backend --target $target_triple --all-features --release"
}