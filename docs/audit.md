# Audit

Manta can log user's operations in `/var/log/manta/` (Linux) or `${PWD}` (MacOS), please make sure this folder exists and the current user has `rwx` access to it

```bash
mkdir /var/log/manta
chmod 777 -R /var/log/manta
```

