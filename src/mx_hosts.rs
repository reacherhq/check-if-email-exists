extern crate trust_dns_resolver;

use self::trust_dns_resolver::config::*;
use self::trust_dns_resolver::lookup::MxLookup;
use self::trust_dns_resolver::Resolver;

pub fn get_mx_lookup(domain: &str) -> MxLookup {
	// Construct a new Resolver with default configuration options
	let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

	// Lookup the MX records associated with a name.
	// The final dot forces this to be an FQDN, otherwise the search rules as specified
	// in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
	resolver.mx_lookup(domain).unwrap()
}
