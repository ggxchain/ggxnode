# General

The stress tester utilizes [artillery](https://www.artillery.io/) framework with substrate [plugin](https://github.com/dwellir-public/artillery-engine-substrate).

Unfortunately, plugin was outdated, so I had to bump it up in [PR](https://github.com/dwellir-public/artillery-engine-substrate/pull/14),
but the team promised to review and release a new version.

## How to

### Prerequisites

```bash
npm install -g artillery

# After PR merge and release, the plugin will be installed by npm install -g artillery-engine-substrate
git clone https://github.com/akorchyn/artillery-engine-substrate
cd artillery-engine-substrate
npm install -g
```

### Running

```bash
npm install # install dependencies

artillery run tests/substrate.yml
```

### Possible area of improvements

* Implement EVM stress testing as well, so we can stress node from all directions
