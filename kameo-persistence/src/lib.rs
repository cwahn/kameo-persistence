pub mod bi_hash_map;
pub mod persistent_actor;

// Re-export local modules
pub use bi_hash_map::BiHashMap;
pub use persistent_actor::PersistentActor;

// Re-export macros
pub use kameo_persistence_macros::PersistentActor;

// Test module
#[cfg(test)]
mod tests {

    #[test]
    fn build() {
        let t = trybuild::TestCases::new();
        t.pass("tests/derive_persistent_actor.rs");
        t.pass("tests/derive_persistent_actor_with_custom_snapshot.rs");
    }
}
