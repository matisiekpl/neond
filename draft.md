- we have a single binary
- it contains a control plane
- neondatabase/neon is connected as a submodule
- this module has control_plane
- there is a single working dir

Modules:

- unpacker
- control_plane

preflight::check()

- check if ports are free
- check if there is enough storage

unpacker::unpack()

- create /tmp/neon/binaries and unloads binaries there
- create /tmp/neon/pg_install and unloads postgreses there

daemon::start()

- put pageserver's identity.toml
- put pageserver.toml config
- put pageserver's metadata.json
- start a fake http server on port 50052 that acts as control plane
- start storage_broker
- start storage_controller postgres
- start safekeeper
- start pageserver

daemon::stop()

- mgmt::login
- mgmt::register
- mgmt::list_organizations
- mgmt::create_organization
- mgmt::update_organization
- mgmt::delete_organization
- mgmt::assign
- mgmt::revoke
- mgmt::list_projects
- mgmt::create_project
- mgmt::update_project
- mgmt::delete_project
- mgmt::list_branches
- mgmt::start(branch)
- mgmt::stop(endpoint)

### Models:

User:

- id
- name
- email
- password_hash

Organization:

- id
- name

Membership:

- user_id
- organization_id

Project:
- id (tenant_id)
- organization_id

Branch:

- id (timeline_id)
- name
- parent_branch_id

Endpoint:

- branch_id
- state (stopped/running)
- endpoint_port


Commands:
```bash
./storage_broker -l  127.0.0.1:50051

./pageserver -D .neon/pageserver_1

./storage_controller -l 127.0.0.1:1234 --database-url "postgresql://neondb_owner:npg_lJCj04edKair@ep-divine-butterfly-alja3qky-pooler.c-3.eu-central-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require" --dev    --timeline-safekeeper-count 1 --control-plane-url https://5083-149-156-124-2.ngrok-free.app

./safekeeper -D .neon/safekeepers/sk1 \
--id 1 \
--broker-endpoint http://127.0.0.1:50051 \
--listen-pg 127.0.0.1:5454 \
--listen-http 127.0.0.1:7676 \
--availability-zone sk-1

./compute_ctl \    
--pgdata .neon/compute_1 \
--pgbin ./../pg_install/v17/bin/postgres \
--compute-id "manual-compute" \
--connstr "postgresql://no_user@localhost:55432/postgres" \
--config compute_1_config.json
```

# Architecture Overview

## `ComputeEndpoint` Structure

- Manages a subprocess with Postgres
- Internally runs `compute_ctl`
- Generates keys
- Accepts a `Branch` as an argument
- Has methods such as `.launch`, `.shutdown`, `.get_status`
- Generates a random port

## `sni_multiplexer` Component

Has methods such as:
- `add_mapping`
- `remove_mapping`
- `swap_mapping`
- `break_connection`

## Startup Flow

1. A request comes to the controller, with a branch ID passed as an argument
2. The service starts a `compute_endpoint` and stores a pointer to it
3. The service registers a mapping in the multiplexer