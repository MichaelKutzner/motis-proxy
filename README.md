# motis-proxy

A very basic proxy to route requests to different backend servers.


## Motivation

For routing, there is an obvious correlation between search space and response times.
Using a smaller instance, either in time or space, should therefore yield faster responses.

As most requests will most likely relate to the next days, reducing the time dimension is quite simple, without having any of the drawbacks, when clipping the area.
This however limits the possibilities, as these instances cannot be used for planning a trip weeks in advance.
Therefore multiple instances are needed.

To keep a coherent user experience, this `motis-proxy` supports multiple backends.
Depending on the `time` parameter passed, the proxy will decide which instance will handle the request.
If the `time` parameter is missing, the fastest instance will be used.
If the query contains the `pageCursor` parameter, the proxy will always select the largest instance instead.


## Build

To build the binary run

```sh
cargo build --release
```

The binary is located at `target/release/motis-proxy`

For development the `debug` build can be used:

```sh
cargo run
```


## Usage

To configure the proxy, use these environment variables:

* `BACKEND_ADDRESS`: Address of the default backend. Default: `http://127.0.0.1:8080`
* `BACKENDS`: List of backend servers with number of days each. Format: `<days1>#<backend1>;<days2>#<backend2>;<days3>#<backend3>…`
* `BIND_ADDR`: IPv4 bind. Default: `0.0.0.0`
* `BIND_PORT`: Bind port. Default: `5173`
* `PROXY_PREFIX`: Optional prefix, that will be removed before passing to the backend. This can be used when the service is not served on the root path and for debugging.

### Example

```sh
BACKEND_ADDRESS=http://127.0.0.1:8083 BACKENDS=3#http://127.0.0.1:8081;9#http://127.0.0.1:8082 BIND_PORT=8080 target/release/motis-proxy
```

This starts a proxy server with backends for 3 and 9 days and the default backend.


## TODO

* Currently all static data will be served by the fastest instance, as the requests don't contain a `time` parameter
    * Option a) Spread these requests
    * Option b) Don't serve tiles on all but one instance
* To cover trips around midnight, the proxy requires backend servers to have at least one day after the requested time
    * For search by arrival, this is not needed
    * An instance with a 8 day schedule can be used for today and the upcoming 6 days. Requests 7 days in advance will be sent to the next larger instance.
    * Assuming a travel duration of less than `x` hours, that threshold (currently 1 day = 24 hours) should be configurable

