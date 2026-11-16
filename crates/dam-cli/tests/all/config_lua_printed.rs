use crate::support;

use support::sandbox::Sandbox;

const PRINTED: &str = "PRINTED-BY-CONFIG";
const WAYS: [&str; 4] = ["print", "write", "stdout", "output"];

fn a_config_that_prints_every_way() -> Sandbox {
    let sb = Sandbox::new();
    let config = std::fs::read_to_string(sb.config_path()).unwrap();
    sb.write_config_lua(&format!(
        "print(\"{PRINTED}-print\")\nio.write(\"{PRINTED}-write\\n\")\n\
         io.stdout:write(\"{PRINTED}-stdout\\n\")\nio.output():write(\"{PRINTED}-output\\n\")\n\
         {config}"
    ));
    sb
}

#[test]
fn what_config_lua_prints_goes_to_standard_error_and_leaves_json_whole() {
    let _guard =
        support::guard("what_config_lua_prints_goes_to_standard_error_and_leaves_json_whole");
    let sb = a_config_that_prints_every_way();
    for line in [
        &["list", "-o", "json"][..],
        &["status", "-o", "json"],
        &["config", "list", "-o", "json"],
        &["doctor", "config", "-o", "json"],
    ] {
        let out = sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        serde_json::from_slice::<serde_json::Value>(&out.stdout)
            .unwrap_or_else(|e| panic!("{line:?}: {e}: {out:?}"));
        let stderr = String::from_utf8_lossy(&out.stderr);
        for way in WAYS {
            assert!(
                stderr.contains(&format!("{PRINTED}-{way}\n")),
                "{line:?} {way}: {stderr}"
            );
        }
    }
}
