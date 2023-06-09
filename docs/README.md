# MongoDB example
[![CI](https://github.com/SergioGasquez/mongodb-example/actions/workflows/CI.yml/badge.svg)](https://github.com/SergioGasquez/mongodb-example/actions/workflows/CI.yml)

Simple example built for [esp-rust-board](https://github.com/esp-rs/esp-rust-board) that sends temperature and humidity to MongoDB, using POST method.

![MongoDB Collection](assets/collection.png)

To reproduce the example
1. Setup MongoDB:
   1. Create an application
   2. Create a database
   3. Create a collection
2. Rename `cfg.toml.example` to `cfg.toml`
3. Fill the `cfg.toml` with:
   1. `wifi_ssid`: Wifi SSID
   2. `wifi_pass`: Wifi password
   3. `api_key`: MongoDB API Key
   4. `data_source`: MongoDB Data Source
   5. `database`: MongoDB Database name
   6. `collection`: MongoDB Collection name
   7. `app_id`: MongoDB App ID

## Testing MongoDB from terminal
To publish data into your collection using `curl`:
```
curl https://data.mongodb-api.com/app/<app_id>/endpoint/data/v1/action/insertOne \
    -H 'Content-Type: application/json' \
    -H 'api-key: <api_key>' \
    --data-raw \
    '{
  "dataSource": "<data_source>",
  "database" : "<data_base>",
  "collection" : "<collection>",
  "document" : { "name": "Harvest",
                 "breed": "Labrador",
                 "age": 5 }
}'
```
For more information, see [MongoDB documentation](https://www.mongodb.com/docs/atlas/api/data-api/#3.-send-a-data-api-request)


## Dev Containers
This repository offers Dev Containers supports for:
-  [VS Code Dev Containers](https://code.visualstudio.com/docs/remote/containers#_quick-start-open-an-existing-folder-in-a-container)
-  [GitHub Codespaces](https://docs.github.com/en/codespaces/developing-in-codespaces/creating-a-codespace)
> **Note**
>
> In [order to use GitHub Codespaces](https://github.com/features/codespaces#faq)
> the project needs to be published in a GitHub repository and the user needs
> to be part of the Codespaces beta or have the project under an organization.

If using VS Code or GitHub Codespaces, you can pull the image instead of building it
from the Dockerfile by selecting the `image` property instead of `build` in
`.devcontainer/devcontainer.json`. Further customization of the Dev Container can
be achived, see [.devcontainer.json reference](https://code.visualstudio.com/docs/remote/devcontainerjson-reference).

When using Dev Containers, some tooling to facilitate building, flashing and
simulating in Wokwi is also added.
### Build
- Terminal approach:

    ```
    scripts/build.sh  [debug | release]
    ```
    > If no argument is passed, `release` will be used as default


-  UI approach:

    The default build task is already set to build the project, and it can be used
    in VS Code and GitHub Codespaces:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Build Task` command.
    - `Terminal`-> `Run Build Task` in the menu.
    - With `Ctrl-Shift-B` or `Cmd-Shift-B`.
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build`.
    - From UI: Press `Build` on the left side of the Status Bar.

### Flash

> **Note**
>
> When using GitHub Codespaces, we need to make the ports
> public, [see instructions](https://docs.github.com/en/codespaces/developing-in-codespaces/forwarding-ports-in-your-codespace#sharing-a-port).

- Terminal approach:
  - Using `flash.sh` script:

    ```
    scripts/flash.sh [debug | release]
    ```
    > If no argument is passed, `release` will be used as default

- UI approach:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build & Flash`.
    - From UI: Press `Build & Flash` on the left side of the Status Bar.
- Any alternative flashing method from host machine.


### Wokwi Simulation

#### VS Code Dev Containers and GitHub Codespaces

The Dev Container includes the Wokwi Vs Code installed, hence you can simulate your built projects doing the following:
1. Press `F1`
2. Run `Wokwi: Start Simulator`

> **Note**
>
>  We assume that the project is built in `debug` mode, if you want to simulate projects in release, please update the `elf` and  `firmware` proprieties in `wokwi.toml`.

For more information and details on how to use the Wokwi extension, see [Getting Started] and [Debugging your code] Chapter of the Wokwi documentation.

[Getting Started]: https://docs.wokwi.com/vscode/getting-started
[Debugging your code]: https://docs.wokwi.com/vscode/debugging
