| bump | sets the version in `version.toml` ([Suffix scheme](agent-data/versioning.md#suffix-scheme)) | first, at each commit of a cycle | edit `version.toml` |
| validate | checks the config and the markdown's links | before each review, and before every commit | `vc-x1 check config`, then `vc-x1 check anchors` |
| validate fast | checks the config | while a review iterates | `vc-x1 check config` |
