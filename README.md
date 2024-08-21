# Motivation
This is mostly for local development where webhooks could not reach localhost. Usually localhost would use something like a Relay server or tunneling to expose local ports to the internet. It works, but there are some networks that you just do not want exposed :).
So, the idea is to use XHR redirects to send the webhooks to localhost. While, this could still work by just redirecting back to the main localhost environment, there are still some pain points that needs to be considered, such as CORS. If possible we want to make the dev environment to be
as close as possible to production and have better isolation between processes and networks.

Enter Interceder, while it still sucks, it gets the job done. You can have it hosted inside a container for somewhat better isolation, or heck add a reverse proxy then connect it to a Relay server. Just make sure
that it is always trusted locally. Interceder will then sit between both networks and delegate the webhooks to where it should actually go.

# Usage

## Manifest

`./config/manifest.toml` must be present to run the server
```toml
[server]
# host ip, use 0.0.0.0 to make server accessible using docker
address = "0.0.0.0"
# host port, make sure to publish port if using docker
port = "3000"
# envs that must exist during runtime
env = [
    "ENV_1",
    "ENV_2",
    "ENV_3"
]

[webhook]
# make sure DNS can reach the domain, or use IP
# if using https:// make sure cert is trusted, as interceder does not accept invalid certs
url = "http://host.docker.internal/path/to/webhook"
# params defined using envs to append to webhook url
# i.e; http://host.docker.internal/path/to/webhook/1
params = ["ENV_1"]
# usually webhooks have topics/events associated with it, use this as a filter
topics = [
    "topic1",
    "topic2",
    "topic3",
    "topic4"
]
# request headers to be added/redirected can be set here
# accept and content-type has already been set to application/json as an opionated config
# topic from the subscribed webhook is also retrieved from here, before comparing with the topics config
# set as req for the header to get from request, set as anything else, to set as env
headers = [
    ["env1", "ENV_1"],
    ["env2", "ENV_2"],
    ["domain", "req"],
    ["topic", "req"]
]
# if webhook provides a hash in the header for verification, it can be set here
# set is_required = false, header = "", if not applicable
hash = { is_required = true, header = "hash" }
# if rehasing is required i.e; local is set differently from production, it can be done here
# currently only suports HMAC SHA256
# the key for the hash will use the value from env
rehash = { is_required = true, secret = "ENV_3" }
```

## Start the server

using Docker
> use -v /path/to/save-payload:/app/payload to have the payload directory mounted on local
```bash
docker run --rm -d \
--name interceder \
-h interceder \
-e ENV_1=$ENV_1 \
-e ENV_2=$ENV_2 \
-e ENV_3=$ENV_3 \
--mount type=bind,source=/path/to/manifest.toml,target=/app/config/manifest.toml,readonly \
ghcr.io/noseferatuwkf/interceder:latest
```

or build from source
```bash
cargo build -r . && ./target/release/interceder
```

## Endpoints

`POST` /intercede
```bash
# redirect to local receiver, this should be called by the subscribed webhook
curl http://localhost:3000/intercede \
-H "accept: application/json"  \
-H "content-type: application/json" \
-H "domain: my.domain.com" \
-H "topic: topic3" \
-d '{"content": "hello"}'
```

`GET` /replay
```bash
# replay last webhook payload
# the headers will need to be set manually
# the last webhook payload is written to file at ./payload/<topic>.json and can be modified
curl http://localhost:3000/replay \
-H "accept: application/json"  \
-H "content-type: application/json" \
-H "domain: my.domain.com" \
-H "topic: topic3"
```

## Payload Cache

every time `/intercede` is called, the payload body will be written to file. This behavior cannot be disabled currently,
and the reason it does this, is to enable intercede to replay the message without relying on new messages, this
also helps persist time sensitive information. The reason why it is written to file instead of memory, is to
enable the modification of the payload before replaying if needed.

`./payload` contains all the payload files, also note that for topic `topic` and `topic/sub-topic` will write to the same file

# TODO
- [ ] tests
