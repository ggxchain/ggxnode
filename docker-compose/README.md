#### Run node with docker-compose

Instantly deployment using prebuild images

**Requirements:**
* [Docker](https://docs.docker.com/engine/install/)
* [Docker Compose](https://docs.docker.com/compose/)
* sudo apt-get install git

_Please prioritize the security of your deployment. Security is beyond the scope of this guidance here, ensure your firewall is properly configured. Be aware that Docker may bypass iptables rules. Make sure you fully understand the implications of the setup._

**What we about to do:**
1) Change the node name in the config `nano <chain>.env`
2) Activate config `cp <chain>.env .evn`
3) Create aprorpriate data folder structure `./run-upgrade.sh`
4) Start node `./run-upgrade.sh`

**Assume we are on Linux. `ggx_user` will be userd here as for example.**

```bash
# create dedicated non-sudo user:
sudo adduser --gecos GECOS --disabled-password --disabled-login --shell /bin/bash ggx_user
```
```bash
# add user in to docker group:
sudo usermod -aG docker ggx_user
```
```bash
# get user shell
su - ggx_user
```
```bash
# clone ggxnode repository in to user home
cd ~ && git clone https://github.com/ggxchain/ggxnode.git
```
```bash
# Copy required folder in to user ${HOME}
cd ~ && cp -r ggxnode/docker-compose/ .
```
* Full repository can be removed now
```bash
rm -rf ${HOME}/ggxnode
```
**Depends on the network we about to join, edit environment files:**

-- for `sydney` edit `sydney.env`
-- for `brooklyn` edit `brooklyn.env`

**All we are looking here is this part:**
```
# -------------------- #
NODE_NAME=my_node_name
# -------------------- #
```

**Guidelines for Naming Variables:**

* _Descriptive Names: use names that clearly indicate the variable's purpose, like `oliver_node`, `dev_node_one`, or `a3mc_little_rpc`._

* _No White Spaces or Special Characters: avoid using spaces or special characters (e.g., @, #, $, %). Use underscores `_` to separate words._

* _Consistent Usage: the variable name will also serve as a data storage folder name and telemetry moniker for consistency across systems._

**Adjust ports if requires or levae them by default**
```bash
P2P_PORT=5000
RPC_PORT=5001
PROMETHEUS_PORT=5002
```
If you plan to make validator out of this node keep `RPC_METHODS=unsafe`, this will allow us to generate session keys using open RPC _( change later )_, otherwise set to `RPC_METHODS=safe`.

Default pruning is `256` change if required or set to `archive`

```bash
STATE_PRUNING=256
BLOCKS_PRUNING=256
```

`SYNC_MODE=warp` require 3 nodes to be online _( which is not always a brooklyn case )_ however on Sydney `warp` will be your best choice.

**Activate network environment**

**Sydney:**
```bash
# make sydney config active
cp sydney.env .env
```

**Brooklyn:**
```bash
# make brooklyn config active
cp brooklyn.env .env
```

**For security reasons, the internal container user ID and group ID are both set to 55500. This requires creating a special data folder owned by user 55500 and accessible by group 55500.**

Script `run-upgrade.sh` will guide you how to create this folder, it will require `sudo` permission.

```bash
# make script executable
chmod +x run-upgrade.sh
```
```bash
# get data folder creation guide
./run-upgrade.sh
```
Copy suggested code, `exit` non-sudo user shell and execute command. Then comeback to our user `su - ggx_user`.

At this point we should have data folder set
```
.
└──data
    └── <node_name>
```

And data folder permissions configured accordingly

```bash
# check permissions
ls -lah ${HOME}/data
```
**Start the node !**
```bash
# Start node
cd ${HOME}/docker-compose && ./run-upgrade.sh
```

Log should populate console, press `Ctrl+C` to exit.