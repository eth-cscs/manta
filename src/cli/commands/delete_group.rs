pub async fn exec(backend: &StaticBackendDispatcher, auth_token: &str, label: &str) {
    // Validate if group can be deleted
    validation(backend, auth_token, label).await;

    // Delete group
    let result = backend.delete_group(auth_token, label).await;

    match result {
        Ok(_) => {
            println!("Group '{}' deleted", label);
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
