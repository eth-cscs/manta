use crate::cli::commands::{
    migrate_backup, migrate_nodes_between_hsm_groups, migrate_restore,
};
use crate::common::{
    app_context::AppContext, authentication::get_api_token,
    authorization::get_groups_names_available,
};
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta migrate` subcommands (nodes, backup,
/// restore).
pub async fn handle_migrate(
    cli_migrate: &ArgMatches,
    ctx: &AppContext<'_>,
) -> Result<(), Error> {
    let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

    if let Some(cli_migrate_nodes) = cli_migrate.subcommand_matches("nodes") {
        let dry_run: bool = cli_migrate_nodes.get_flag("dry-run");
        let from_opt = cli_migrate_nodes
            .get_one::<String>("from")
            .map(String::as_str);
        let to: &str = cli_migrate_nodes
            .get_one::<String>("to")
            .map(String::as_str)
            .context("The 'to' argument is mandatory")?;
        let xnames_string: &String = cli_migrate_nodes
            .get_one("XNAMES")
            .context("The 'XNAMES' argument must have a value")?;
        let from = get_groups_names_available(
            ctx.infra.backend,
            &token,
            from_opt,
            ctx.cli.settings_hsm_group_name_opt,
        )
        .await?;
        let to = get_groups_names_available(
            ctx.infra.backend,
            &token,
            Some(to),
            ctx.cli.settings_hsm_group_name_opt,
        )
        .await?;
        migrate_nodes_between_hsm_groups::exec(
            ctx,
            &token,
            &to,
            &from,
            xnames_string,
            dry_run,
            false,
        )
        .await?;
    } else if let Some(_cli_migrate_vcluster) =
        cli_migrate.subcommand_matches("vCluster")
    {
        if let Some(cli_migrate_vcluster_backup) =
            cli_migrate.subcommand_matches("backup")
        {
            let bos: Option<&String> =
                cli_migrate_vcluster_backup.get_one("bos");
            let destination: Option<&String> =
                cli_migrate_vcluster_backup.get_one("destination");
            let prehook: Option<&String> =
                cli_migrate_vcluster_backup.get_one("pre-hook");
            let posthook: Option<&String> =
                cli_migrate_vcluster_backup.get_one("post-hook");
            migrate_backup::exec(
                ctx,
                &token,
                bos.map(String::as_str),
                destination.map(String::as_str),
                prehook.map(String::as_str),
                posthook.map(String::as_str),
            )
            .await?;
        } else if let Some(cli_migrate_vcluster_restore) =
            cli_migrate.subcommand_matches("restore")
        {
            let bos_file: Option<&String> =
                cli_migrate_vcluster_restore.get_one("bos-file");
            let cfs_file: Option<&String> =
                cli_migrate_vcluster_restore.get_one("cfs-file");
            let hsm_file: Option<&String> =
                cli_migrate_vcluster_restore.get_one("hsm-file");
            let ims_file: Option<&String> =
                cli_migrate_vcluster_restore.get_one("ims-file");
            let image_dir: Option<&String> =
                cli_migrate_vcluster_restore.get_one("image-dir");
            let prehook: Option<&String> =
                cli_migrate_vcluster_restore.get_one("pre-hook");
            let posthook: Option<&String> =
                cli_migrate_vcluster_restore.get_one("post-hook");
            let overwrite: bool =
                cli_migrate_vcluster_restore.get_flag("overwrite");
            migrate_restore::exec(
                ctx,
                &token,
                bos_file.map(String::as_str),
                cfs_file.map(String::as_str),
                hsm_file.map(String::as_str),
                ims_file.map(String::as_str),
                image_dir.map(String::as_str),
                prehook.map(String::as_str),
                posthook.map(String::as_str),
                overwrite,
            )
            .await?;
        } else {
            bail!("Unknown 'migrate vCluster' subcommand");
        }
    } else {
        bail!("Unknown 'migrate' subcommand");
    }
    Ok(())
}
