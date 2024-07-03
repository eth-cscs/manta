# Audit

Manta can log user's operations in a local file. The audit file location can be specified in config file with the `audot_config` field. If missing, then the default location is `/var/log/manta/` (Linux). Make sure this folder exists and the current user has `rwx` access to it

```bash
mkdir /var/log/manta
chmod 777 -R /var/log/manta
```

