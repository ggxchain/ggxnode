# General

The stress tester utilizes [artillery](https://www.artillery.io/) framework with substrate [plugin](https://github.com/dwellir-public/artillery-engine-substrate).

## How to

### Prerequisites

```bash
npm install
```

### Running

```bash
npm run artillery
```

### Generating report

```bash
npm run artillery-report
```

### Possible area of improvements

* Implement EVM stress testing as well, so we can stress the node from all directions
* Reports are useless because our scenario setup can fail, so we skip processing and instantly finish the run.
Unfortunately, this can't be handled because the Substrate plugin API prevents you from failing.
The `done` callback carries success. Though it's easy to do for them.
