//! Bundled built-in skill files, embedded at compile time.
//! These are written to `skills/` during `dalang init`.

pub struct BundledSkill {
    pub filename: &'static str,
    pub content: &'static str,
}

pub const BUNDLED_SKILLS: &[BundledSkill] = &[
    BundledSkill {
        filename: "nmap_scanner.md",
        content: include_str!("../../../../skills/nmap_scanner.md"),
    },
    BundledSkill {
        filename: "masscan_fast.md",
        content: include_str!("../../../../skills/masscan_fast.md"),
    },
    BundledSkill {
        filename: "rustscan_audit.md",
        content: include_str!("../../../../skills/rustscan_audit.md"),
    },
    BundledSkill {
        filename: "ffuf_fuzzer.md",
        content: include_str!("../../../../skills/ffuf_fuzzer.md"),
    },
    BundledSkill {
        filename: "sqlmap_tester.md",
        content: include_str!("../../../../skills/sqlmap_tester.md"),
    },
    BundledSkill {
        filename: "xss_strike.md",
        content: include_str!("../../../../skills/xss_strike.md"),
    },
    BundledSkill {
        filename: "web-audit.md",
        content: include_str!("../../../../skills/web-audit.md"),
    },
    BundledSkill {
        filename: "nikto_scanner.md",
        content: include_str!("../../../../skills/nikto_scanner.md"),
    },
    BundledSkill {
        filename: "wpscan_audit.md",
        content: include_str!("../../../../skills/wpscan_audit.md"),
    },
    BundledSkill {
        filename: "hydra_bruteforce.md",
        content: include_str!("../../../../skills/hydra_bruteforce.md"),
    },
    BundledSkill {
        filename: "smbclient_enum.md",
        content: include_str!("../../../../skills/smbclient_enum.md"),
    },
    BundledSkill {
        filename: "snmpwalk_gather.md",
        content: include_str!("../../../../skills/snmpwalk_gather.md"),
    },
    BundledSkill {
        filename: "kubectl_audit.md",
        content: include_str!("../../../../skills/kubectl_audit.md"),
    },
    BundledSkill {
        filename: "aws_cli_enum.md",
        content: include_str!("../../../../skills/aws_cli_enum.md"),
    },
    BundledSkill {
        filename: "docker_escape_check.md",
        content: include_str!("../../../../skills/docker_escape_check.md"),
    },
    BundledSkill {
        filename: "testing.md",
        content: include_str!("../../../../skills/testing.md"),
    },
    BundledSkill {
        filename: "header_analyzer.md",
        content: include_str!("../../../../skills/header_analyzer.md"),
    },
    BundledSkill {
        filename: "ssl_scan.md",
        content: include_str!("../../../../skills/ssl_scan.md"),
    },
    BundledSkill {
        filename: "jwt_analysis.md",
        content: include_str!("../../../../skills/jwt_analysis.md"),
    },
    BundledSkill {
        filename: "nuclei_vuln_scan.md",
        content: include_str!("../../../../skills/nuclei_vuln_scan.md"),
    },
    BundledSkill {
        filename: "subdomain_enum.md",
        content: include_str!("../../../../skills/subdomain_enum.md"),
    },
    BundledSkill {
        filename: "gobuster_dir.md",
        content: include_str!("../../../../skills/gobuster_dir.md"),
    },
];
