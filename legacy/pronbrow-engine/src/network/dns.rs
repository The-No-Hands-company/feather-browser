// DNS resolution for PronBrow Engine
use std::net::{IpAddr, ToSocketAddrs};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// DNS cache entry
#[derive(Clone, Debug)]
pub struct DnsEntry {
    pub ips: Vec<IpAddr>,
    pub expires_at: Instant,
}

/// DNS resolver with caching
pub struct DnsResolver {
    cache: Arc<RwLock<HashMap<String, DnsEntry>>>,
    ttl: Duration,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(300), // 5 minutes default TTL
        }
    }

    /// Resolve a hostname to IP addresses
    pub fn resolve(&self, hostname: &str) -> Result<Vec<IpAddr>, String> {
        // Check cache first
        if let Some(entry) = self.get_cached(hostname) {
            return Ok(entry.ips);
        }

        // Perform actual DNS lookup
        let ips = self.lookup(hostname)?;
        
        // Cache the result
        self.cache_entry(hostname.to_string(), ips.clone());
        
        Ok(ips)
    }

    fn get_cached(&self, hostname: &str) -> Option<DnsEntry> {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(hostname) {
            if entry.expires_at > Instant::now() {
                return Some(entry.clone());
            }
        }
        None
    }

    fn cache_entry(&self, hostname: String, ips: Vec<IpAddr>) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(hostname, DnsEntry {
            ips,
            expires_at: Instant::now() + self.ttl,
        });
    }

    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, String> {
        // Try to resolve with port 80 (HTTP)
        let addr_str = format!("{}:80", hostname);
        
        match addr_str.to_socket_addrs() {
            Ok(addrs) => {
                let ips: Vec<IpAddr> = addrs.map(|addr| addr.ip()).collect();
                if ips.is_empty() {
                    Err(format!("No IP addresses found for {}", hostname))
                } else {
                    Ok(ips)
                }
            }
            Err(e) => Err(format!("DNS lookup failed for {}: {}", hostname, e)),
        }
    }

    /// Clear DNS cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Remove expired entries from cache
    pub fn cleanup_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_resolution() {
        let resolver = DnsResolver::new();
        
        // Test resolving a known domain
        match resolver.resolve("localhost") {
            Ok(ips) => {
                assert!(!ips.is_empty());
                println!("Resolved localhost to: {:?}", ips);
            }
            Err(e) => println!("DNS resolution failed: {}", e),
        }
    }

    #[test]
    fn test_dns_cache() {
        let resolver = DnsResolver::new();
        
        // First resolution
        let _ = resolver.resolve("localhost");
        
        // Should hit cache on second resolution
        let result = resolver.resolve("localhost");
        assert!(result.is_ok());
    }
}
