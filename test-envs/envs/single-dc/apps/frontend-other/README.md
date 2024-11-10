
# Eden Platform frontend app

## Requirements

- Nix package manager with [flakes support](https://nixos.wiki/wiki/Flakes)
- Docker

## Develop with trunk

### Enter the nix shell environment

```
nix develop
```

### Build trunk app

```
trunk build
```

### Serve trunk app

`--public-url` is needed because Trunk.toml uses relative path for building.
```
trunk serve --public-url=/
```

### Serve trunk app with possibly live backend
```
trunk serve --public-url=/ --proxy-backend https://www.epl-infra.net/api/
```

more trunk documentation can be found [here](https://trunkrs.dev/configuration/)

## Build the project with running tests

```
nix build
```

## Loading and running docker image

Should build a docker image file named `result`.

You can load docker image locally
```
docker load -i result
```

Then run the image on custom port 12421
```
docker run --rm -e EPL_HTTP_SOCKET=0.0.0.0:12421 -p 127.0.0.1:12421:12421 -it frontend-test:v0.1.0-l9z821n112ria1hv5w1hyl3zdwgp9xby
```

Page should be available in the browser address http://127.0.0.1:12421/


