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

artillery run --output report.json tests/substrate.yml
```

### Generating report

```bash
artillery report report.json
```

### Possible area of improvements

* Implement EVM stress testing as well, so we can stress the node from all directions
* Reports are useless because our scenario setup can fail, so we skip processing and instantly finish the run.
Unfortunately, this can't be handled because the Substrate plugin API prevents you from failing.
The `done` callback carries success. Though it's easy to do for them.
