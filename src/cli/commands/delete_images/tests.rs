use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use manta_backend_dispatcher::types::BootParameters;

/* #[test]
fn test_error_if_deleting_image_used_to_boot() {
  let backend = StaticBackendDispatcher::new("csm", "", &[]);

  let boot_parameter = BootParameters {
    hosts: vec!["x1".to_string()],
    macs: None,
    nids: Some(vec![1]),
    params: "root=image_id".to_string(),
    kernel: "kernel".to_string(),
    initrd: "initrd".to_string(),
    cloud_init: None,
  };
} */
