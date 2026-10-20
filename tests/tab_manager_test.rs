// SPDX-License-Identifier: AGPL-3.0-or-later
// tests/tab_manager_test.rs — just the import, full tests in Task 6
// This forces the trait + mock to exist and compile
use feather::bridge::{PageRenderer, MockRenderer};

#[test]
fn test_mock_renderer_implements_trait() {
    let r: Box<dyn PageRenderer> = Box::new(MockRenderer::new("https://example.com"));
    assert_eq!(r.current_url(), "https://example.com");
    assert_eq!(r.title(), "Mock Page");
    assert!(!r.can_go_back());
}
