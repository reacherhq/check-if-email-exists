$cross = $args[0]
$target_triple = $args[1]
$release_build = $args[2]

if ( $release_build -ne "RELEASE" ) {
    Invoke-Expression "$cross build --target $target_triple"
    Invoke-Expression "$cross build --target $target_triple --all-features"
} else {
    Invoke-Expression "$cross build --target $target_triple --all-features --release"
}