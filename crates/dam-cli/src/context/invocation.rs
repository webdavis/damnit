pub(crate) fn invocation_of(
    config: &std::path::Path,
    store: &std::path::Path,
) -> dam_application::Invocation {
    let absolute = |p: &std::path::Path| {
        std::path::absolute(p)
            .unwrap_or_else(|_| p.to_path_buf())
            .display()
            .to_string()
    };
    dam_application::Invocation {
        dam: std::env::current_exe().map_or_else(|_| "dam".into(), |p| absolute(&p)),
        config: absolute(config),
        store: absolute(store),
        search_path: std::env::var("PATH").unwrap_or_default(),
    }
}
